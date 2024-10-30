use std::error::Error;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;
use std::collections::HashMap;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::types::UserScore;
use crate::handlers::{command_handler, recursive_callback_handler, start_reminder_sender};
use crate::state::BotState;
use crate::commands::Command;

mod types;
mod commands;
mod handlers;
mod error;
mod state;
mod keyboard;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting trivia bot...");

    // Initialize bot with token from environment
    let bot = Bot::from_env();

    // Load questions from CSV
    let questions = state::load_questions()?;
    log::info!("Loaded {} questions", questions.len());

    // Load reminder templates
    let reminder_templates = state::load_reminder_templates()?;
    log::info!("Loaded {} reminder templates", reminder_templates.len());

    // Load existing user scores
    let user_scores = UserScore::load_scores()?;
    log::info!("Loaded scores for {} users", user_scores.len());

    // Load user preferences
    let user_preferences = match BotState::initialize_preferences().await {
        Ok(prefs) => {
            log::info!("Successfully initialized preferences for {} users", prefs.len());
            prefs
        }
        Err(e) => {
            log::error!("Failed to initialize preferences: {}. Starting with empty preferences.", e);
            HashMap::new()
        }
    };

    // Initialize bot state
    let state = Arc::new(BotState {
        questions,
        active_questions: Mutex::new(HashMap::new()),
        user_scores: Mutex::new(user_scores),
        rng: Mutex::new(StdRng::from_entropy()),
        reminder_templates,
        user_preferences: Mutex::new(user_preferences),
    });

    // Clone bot and state for reminder service
    let reminder_bot = bot.clone();
    let reminder_state = state.clone();

    // Spawn reminder service
    tokio::spawn(async move {
        start_reminder_sender(reminder_bot, reminder_state).await;
    });

    let handler = dptree::entry()
        .branch(Update::filter_message().filter_command::<Command>().endpoint(
            |bot: Bot, msg: Message, cmd: Command, state: Arc<BotState>| async move {
                log::info!("entry trivia bot...");
                command_handler(bot, msg, cmd, state.clone()).await
            },
        ))
        .branch(recursive_callback_handler( state.clone()));
 
     log::info!("Starting command dispatching...");

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
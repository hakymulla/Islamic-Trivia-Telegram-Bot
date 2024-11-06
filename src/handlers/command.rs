use crate::{BotState, Command};
use std::error::Error;
use std::sync::Arc;
use teloxide::prelude::*;
use rand::seq::IteratorRandom;
use crate::types::{ActiveQuestion, GameState};
use crate::keyboard::create_keyboard;
use teloxide::utils::command::BotCommands;

use crate::handlers::*;

pub async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: Arc<BotState>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cmd {
        Command::Start => {
            bot.send_message(
                msg.chat.id,
                "
                \n 🕌 Use /question for a random question to deepen your Islamic knowledge.
                \n 📚 Use /theme <category> for themed quizzes on various topics.
                \n 🏆 Use /leaderboard to see top scores and track your progress.
                \n 🔔 Use /optin to receive daily Islamic and Sunnah reminders (6 times a day) designed to help you build habits through repetition. Sunnah reminders change weekly to keep things fresh and engaging.
                \n 🔕 Use /optout if you prefer not to receive reminders.
                \n ❓ Use /help for additional guidance.
                "
            )
            .await?;
        }
        Command::Question => {
            start_new_quiz(bot, msg.chat.id, 5, state).await? // Start a 5-question quiz
        }
        Command::Theme(category) => {
            let themed_questions: Vec<_> = state
                .questions
                .iter()
                .filter(|q| q.category.to_lowercase() == category.to_lowercase())
                .collect();

            let question = {
                let mut rng = state.rng.lock().await;
                themed_questions.iter().choose(&mut *rng)
            };

            if let Some(question) = question.cloned() {
                let question = question.clone();
                let sent_message = bot
                    .send_message(msg.chat.id, &question.question)
                    .reply_markup(create_keyboard(&question, None, false, true))
                    .await?;

                state.active_questions.lock().await.insert(
                    msg.chat.id.0,
                    ActiveQuestion {
                        question: question.clone(),
                        message_id: sent_message.id,
                        game_state: GameState::Ended,
                    },
                );
            } else {
                bot.send_message(msg.chat.id, "No questions found for this category!")
                    .await?;
            }
        }
        Command::Leaderboard => {
            let scores = state.user_scores.lock().await;
            let mut scores: Vec<_> = scores.values().collect();
            scores.sort_by(|a, b| b.score.cmp(&a.score));
            
            let leaderboard = scores
                .iter()
                .take(10)
                .enumerate()
                .map(|(i, user)| format!("{}. {} - {} points", i + 1, user.username, user.score))
                .collect::<Vec<_>>()
                .join("\n");
                
            bot.send_message(msg.chat.id, format!("🏆 Leaderboard:\n\n{}", leaderboard))
                .await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::OptIn => {
            handle_opt_in(bot, msg, state).await?;
        }
        Command::OptOut => {
            handle_opt_out(bot, msg, state).await?;
        }
        Command::Preferences => {
            handle_preferences(bot, msg, state).await?;
        }
    }
    Ok(())
}

// Helper function to start a new quiz
async fn start_new_quiz(
    bot: Bot,
    chat_id: ChatId,
    max_questions: u32,
    state: Arc<BotState>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let question = {
        let mut rng = state.rng.lock().await;
        state.questions.iter().choose(&mut *rng).unwrap().clone()
    };

    let sent_message = bot
        .send_message(chat_id, &format!("Question 1/{}\n\n{}", max_questions, question.question))
        .reply_markup(create_keyboard(&question, None, false, true))
        .await?;

    state.active_questions.lock().await.insert(
        chat_id.0,
        ActiveQuestion {
            question,
            message_id: sent_message.id,
            game_state: GameState::InProgress {
                questions_asked: 1,
                max_questions,
            },
        },
    );

    Ok(())
}

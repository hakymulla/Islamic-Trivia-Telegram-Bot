use crate::BotState;
use crate::types::GameState;
use chrono::Utc;
use rand::seq::IteratorRandom;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::prelude::*;
use teloxide::types::CallbackQuery;
use crate::keyboard::create_keyboard;
use crate::types::{ActiveQuestion, UserScore};

pub fn recursive_callback_handler(
    state: Arc<BotState>,
) -> dptree::Handler<'static, DependencyMap, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> 
{
    Update::filter_callback_query()
        .endpoint(move |bot: Bot, q: CallbackQuery| {
            let state = state.clone();
            async move {
                handle_callback_query(bot, q, state).await
            }
        })
}

// Modified handle_callback_query to persist scores
pub async fn handle_callback_query(
    bot: Bot,
    query: CallbackQuery,
    state: Arc<BotState>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let (Some(message), Some(data)) = (query.message, query.data) {
        let chat_id = message.chat.id;
        
        if data == "end_quiz" {
            let mut active_questions = state.active_questions.lock().await;
            if active_questions.remove(&chat_id.0).is_some() {
                let scores = state.user_scores.lock().await;
                if let Some(user_score) = scores.get(&chat_id.0) {
                    bot.send_message(
                        chat_id,
                        format!("Quiz ended! Your final score: {} points", user_score.score),
                    )
                    .await?;
                }
                return Ok(());
            }
        }

        let current_question = {
            let active_questions = state.active_questions.lock().await;
            active_questions.get(&chat_id.0).cloned()
        };

        if let Some(active_question) = current_question {
            let question = &active_question.question;
            let is_correct = data == question.correct_answer;
            let game_state = active_question.game_state;

            bot.edit_message_reply_markup(chat_id, message.id)
                .reply_markup(create_keyboard(question, Some(&data), true, true))
                .await?;

            if is_correct {
                let mut scores = state.user_scores.lock().await;
                let score = scores.entry(chat_id.0).or_insert(UserScore {
                    user_id: chat_id.0,
                    username: query.from.first_name.clone(),
                    score: 0,
                    last_answer_time: Utc::now(),
                });
                
                score.score += question.points;
                score.last_answer_time = Utc::now();
                
                // Save scores after updating
                drop(scores); // Release the lock before saving
                if let Err(e) = state.save_scores().await {
                    log::error!("Failed to save scores: {}", e);
                }
                
                bot.send_message(
                    chat_id,
                    format!("🎉 Correct! You earned {} points!", question.points),
                )
                .await?;
            } else {
                bot.send_message(chat_id, "❌ Sorry, that's incorrect!")
                    .await?;
            }
                        // Check if we should continue with next question
            if let GameState::InProgress { questions_asked, max_questions } = game_state {
                if questions_asked < max_questions {
                    // Generate next question
                    let next_question = {
                        let mut rng = state.rng.lock().await;
                        state.questions.iter().choose(&mut *rng).unwrap().clone()
                    };

                    // Send next question
                    let sent_message = bot
                        .send_message(
                            chat_id,
                            &format!("Question {}/{}\n\n{}", 
                                    questions_asked + 1, 
                                    max_questions, 
                                    next_question.question)
                        )
                        .reply_markup(create_keyboard(&next_question, None, false, true))
                        .await?;

                    // Update active question with new state
                    let mut active_questions = state.active_questions.lock().await;
                    active_questions.insert(
                        chat_id.0,
                        ActiveQuestion {
                            question: next_question,
                            message_id: sent_message.id,
                            game_state: GameState::InProgress {
                                questions_asked: questions_asked + 1,
                                max_questions,
                            },
                        },
                    );
                } else {
                    // End of quiz
                    let scores = state.user_scores.lock().await;
                    if let Some(user_score) = scores.get(&chat_id.0) {
                        bot.send_message(
                            chat_id,
                            format!("Quiz completed! Your final score: {} points", user_score.score),
                        )
                        .await?;
                    }
                    let mut active_questions = state.active_questions.lock().await;
                    active_questions.remove(&chat_id.0);
                }
            }
        }

        bot.answer_callback_query(query.id).await?;
    }
    Ok(())
}
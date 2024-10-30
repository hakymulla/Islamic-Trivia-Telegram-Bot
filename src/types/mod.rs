use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use teloxide::types::MessageId;

mod reminder;
pub use reminder::*;

#[derive(Clone, PartialEq)]
pub enum GameState {
    InProgress { questions_asked: u32, max_questions: u32 },
    Ended,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Question {
    pub id: u32,
    pub question: String,
    pub correct_answer: String,
    pub option1: String,
    pub option2: String,
    pub option3: String,
    pub option4: String,
    pub category: String,
    pub points: u32,
}

impl Question {
    pub fn get_options(&self) -> Vec<String> {
        vec![
            self.option1.clone(),
            self.option2.clone(),
            self.option3.clone(),
            self.option4.clone(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserScore {
    pub user_id: i64,
    pub username: String,
    pub score: u32,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_answer_time: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ActiveQuestion {
    pub question: Question,
    pub message_id: MessageId,
    pub game_state: GameState,
}
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Deserialize)]
pub struct ReminderTemplate {
    pub message: String,
    pub arabic: String,
    pub transliteration: String,
    pub translation: String,
    pub reference: String,
}


#[derive(Debug, Clone, Deserialize)]
pub struct ReminderTemplateAct {
    pub message: String,
    pub act: String,
    pub reference: String,
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReminderPreferences {
    pub user_id: i64,
    pub username: String,
    pub opted_in: bool,
    pub last_reminder: Option<DateTime<Utc>>,
}

impl UserReminderPreferences {
    pub fn new(user_id: i64, username: String) -> Self {
        Self {
            user_id,
            username,
            opted_in: false,
            last_reminder: None,
        }
    }
}

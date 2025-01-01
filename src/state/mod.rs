use crate::types::{Question, ActiveQuestion, UserScore};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use tokio::sync::Mutex;
use rand::rngs::StdRng;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::error::ScoreError;
use crate::types::{ReminderTemplate, ReminderTemplateAct, UserReminderPreferences};
use tokio::time::timeout;
use std::time::Duration;
use reqwest;
pub struct BotState {
    pub questions: Vec<Question>,
    pub active_questions: Mutex<HashMap<i64, ActiveQuestion>>,
    pub user_scores: Mutex<HashMap<i64, UserScore>>,
    pub rng: Mutex<StdRng>,
    pub reminder_templates: Vec<ReminderTemplate>,
    pub reminder_templates_act: Vec<ReminderTemplateAct>,
    pub user_preferences: Mutex<HashMap<i64, UserReminderPreferences>>,
}

impl BotState {
    pub async fn save_scores(&self) -> Result<(), ScoreError> {
        let scores = self.user_scores.lock().await;
        UserScore::save_scores_async(&scores).await
    }

    pub async fn initialize_preferences() -> Result<HashMap<i64, UserReminderPreferences>, ScoreError> {
        let preferences_path = Path::new("user_preferences.json");
        
        if !preferences_path.exists() {
            log::info!("Creating new user_preferences.json file");
            let empty_prefs: HashMap<i64, UserReminderPreferences> = HashMap::new();
            let json = serde_json::to_string_pretty(&empty_prefs)?;
            let mut file = File::create(preferences_path).await?;
            file.write_all(json.as_bytes()).await?;
            Ok(empty_prefs)
        } else {
            log::info!("Loading existing user_preferences.json");
            let json = fs::read_to_string(preferences_path)?;
            let preferences = serde_json::from_str(&json)?;
            Ok(preferences)
        }
    }

    pub async fn save_preferences(&self) -> Result<(), ScoreError> {
        let preferences = self.user_preferences.lock().await;
        let json = serde_json::to_string_pretty(&*preferences)?;
        
        // Create a temporary file first
        let temp_path = Path::new("user_preferences.tmp.json");
        let mut temp_file = File::create(temp_path).await?;
        temp_file.write_all(json.as_bytes()).await?;
        
        // Rename temporary file to actual file
        tokio::fs::rename(temp_path, "user_preferences.json").await?;
        
        Ok(())
    }

    pub async fn acquire_preferences_lock(&self) -> Result<tokio::sync::MutexGuard<HashMap<i64, UserReminderPreferences>>, Box<dyn Error + Send + Sync>> {
        match timeout(Duration::from_secs(5), self.user_preferences.lock()).await {
            Ok(guard) => Ok(guard),
            Err(_) => {
                log::error!("Timeout while acquiring preferences lock");
                Err("Lock acquisition timeout".into())
            }
        }
    }
}

impl UserScore {
    const SCORES_FILE: &'static str = "user_scores.json";
    
    pub async fn save_scores_async(scores: &HashMap<i64, UserScore>) -> Result<(), ScoreError> {
        let json = serde_json::to_string_pretty(scores)?;
        let mut file = File::create(Self::SCORES_FILE).await?;
        file.write_all(json.as_bytes()).await?;
        Ok(())
    }
    
    pub fn load_scores() -> Result<HashMap<i64, UserScore>, ScoreError> {
        if Path::new(Self::SCORES_FILE).exists() {
            let json = fs::read_to_string(Self::SCORES_FILE)?;
            let scores = serde_json::from_str(&json)?;
            Ok(scores)
        } else {
            Ok(HashMap::new())
        }
    }
}

pub fn load_questions() -> Result<Vec<Question>, Box<dyn Error>> {
    let mut questions = Vec::new();
    let mut rdr = csv::Reader::from_path("questions.csv").expect("question csv failed");
    
    for result in rdr.deserialize() {
        let question: Question = result?;
        questions.push(question);
    }
    Ok(questions)
}

pub async fn load_reminder_templates() -> Result<(Vec<ReminderTemplate>, Vec<ReminderTemplateAct>), Box<dyn Error>> {
    let mut templates = Vec::new();
    let mut templates2 = Vec::new();

    // Make sure to use the correct export URL format
    let url = "https://docs.google.com/spreadsheets/d/e/2PACX-1vRjbtw5iUNyWqrI1sFmAaO4KOxwLuwa_TgJub4b74Uv5uQYYSPl69BIoFBPLQbPhZ5oKn2y2cJrHdIL/pub?gid=0&single=true&output=csv";
    let url2 = "https://docs.google.com/spreadsheets/d/e/2PACX-1vRjbtw5iUNyWqrI1sFmAaO4KOxwLuwa_TgJub4b74Uv5uQYYSPl69BIoFBPLQbPhZ5oKn2y2cJrHdIL/pub?gid=863587672&single=true&output=csv";
    
    let response = reqwest::get(url).await?;
    let content = response.text().await?;

    let response2 = reqwest::get(url2).await?;
    let content2 = response2.text().await?;
    
    // Verify we got CSV and not HTML
    if content.starts_with("<!DOCTYPE html>") {
        return Err("Received HTML instead of CSV Sheet 1. Check if the sheet is properly published and accessible.".into());
    }
    
    if content2.starts_with("<!DOCTYPE html>") {
        return Err("Received HTML instead of CSV Sheet 2. Check if the sheet is properly published and accessible.".into());
    }

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

        let mut rdr2 = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(content2.as_bytes());

    // Print headers to verify structure
    println!("Headers: {:?}", rdr.headers()?);
    println!("Headers: {:?}", rdr2.headers()?);
    
    for (index, result) in rdr.deserialize().enumerate() {
        match result {
            Ok(template) => templates.push(template),
            Err(e) => {
                println!("Error at row {}: {}", index + 1, e);
                continue;
            }
        }
    }

    for (index, result) in rdr2.deserialize().enumerate() {
        match result {
            Ok(template) => templates2.push(template),
            Err(e) => {
                println!("Error at row {}: {}", index + 1, e);
                continue;
            }
        }
    }
    
    if templates.is_empty() {
        return Err("No valid templates found".into());
    }

    if templates2.is_empty() {
        return Err("No valid templates found".into());
    }
    
    Ok((templates, templates2))
}

use crate::BotState;
use chrono::{Datelike, Weekday, Utc};
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::time::{timeout, interval, Duration};
use crate::types::UserReminderPreferences;
use std::error::Error;

pub async fn handle_opt_out(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut preferences = state.user_preferences.lock().await;
    if let Some(prefs) = preferences.get_mut(&msg.chat.id.0) {
        prefs.opted_in = false;
        
        bot.send_message(
            msg.chat.id,
            "✅ You've successfully opted out of reminders. Use /optin anytime to start receiving them again.",
        )
        .await?;
    }

    state.save_preferences().await?;
    Ok(())
}


pub async fn handle_opt_in(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    log::info!("Handling Optin Option...");
    
    // Try to acquire the lock with timeout
    let mut preferences = match state.acquire_preferences_lock().await {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to acquire preferences lock: {}", e);
            bot.send_message(
                msg.chat.id,
                "Sorry, the system is busy. Please try again in a few moments.",
            )
            .await?;
            return Ok(());
        }
    };

    log::info!("Lock acquired successfully");
    
    // Wrap the critical section in a block to ensure the lock is released as soon as possible
    {
        let prefs = preferences.entry(msg.chat.id.0).or_insert_with(|| {
            UserReminderPreferences::new(
                msg.chat.id.0,
                msg.from().map_or("Unknown".to_string(), |u| u.first_name.clone())
            )
        });
        prefs.opted_in = true;
    }
    
    // Release the lock before sending the message and saving preferences
    drop(preferences);
    
    log::info!("Sending confirmation message");
    bot.send_message(
        msg.chat.id,
        "✅ You've successfully opted in to receive daily reminders! You'll receive one random reminder every 24 hours.",
    )
    .await?;

    log::info!("Saving preferences");
    // Add timeout to save_preferences as well
    match timeout(Duration::from_secs(5), state.save_preferences()).await {
        Ok(result) => {
            if let Err(e) = result {
                log::error!("Error saving preferences: {}", e);
                bot.send_message(
                    msg.chat.id,
                    "Warning: There was an issue saving your preferences. Your settings might not persist after bot restart.",
                )
                .await?;
            }
        }
        Err(_) => {
            log::error!("Timeout while saving preferences");
            bot.send_message(
                msg.chat.id,
                "Warning: Saving preferences timed out. Your settings might not persist after bot restart.",
            )
            .await?;
        }
    }

    log::info!("Opt-in handling completed successfully");
    Ok(())
}

pub async fn handle_preferences(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let preferences = state.user_preferences.lock().await;
    
    if let Some(prefs) = preferences.get(&msg.chat.id.0) {
        let status = if prefs.opted_in { "opted in" } else { "opted out" };
        
        bot.send_message(
            msg.chat.id,
            format!(
                "Your reminder preferences:\nStatus: {}\nLast reminder: {}", 
                status,
                prefs.last_reminder
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                    .unwrap_or_else(|| "Never".to_string())
            ),
        )
        .await?;
    } else {
        bot.send_message(
            msg.chat.id,
            "You haven't set any preferences yet. Use /optin to start receiving reminders.",
        )
        .await?;
    }

    Ok(())
}



pub async fn start_reminder_sender(bot: Bot, state: Arc<BotState>) {
    let mut template_sender_id = 0;
    let mut interval = interval(Duration::from_secs(60)); // 1 minute interval
    let mut next_send_time = Utc::now() + Duration::from_secs(5); // First send at 1 minute
    let mut last_monday_check = Utc::now().date_naive();

    loop {
        let now = Utc::now();

        if now >= next_send_time {

            send_reminders(&bot, &state, template_sender_id).await;
            next_send_time = now + Duration::from_secs(60); // Update next send time

            if now.weekday() == Weekday::Mon && now.date_naive() != last_monday_check {
                template_sender_id += 1;
                last_monday_check = now.date_naive(); // Update the last check date
                log::info!("Monday detected: Incremented template_sender_id to {}", template_sender_id);
            }
        }
        interval.tick().await;
    }
}

async fn send_reminders(bot: &Bot, state: &Arc<BotState>, template_sender_id: usize) {
    let preferences = match state.acquire_preferences_lock().await {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to acquire lock in reminder sender: {}", e);
            return;
        }
    };

    let now = Utc::now();

    for (user_id, prefs) in preferences.iter() {
        if !prefs.opted_in {
            continue;
        }

        // Check if at least 1 minute has passed since last reminder
        if let Some(last_reminder) = prefs.last_reminder {
            if (now - last_reminder).num_minutes() < 1 {
                continue;
            }
        }

        // Use the thread-safe RNG instance
        if let Some(template) = state.reminder_templates.get(template_sender_id) {
            // log::info!("This is the id of the reminder template {}", template_sender_id);

            fn escape_markdown_v2(text: &str) -> String {
                text.chars()
                    .map(|c| match c {
                        '_' | '*' | '[' | ']' | '(' | ')' | '~' | '`' | '>' | '#' | '+' | '-' | '=' | '|' | '{' | '}' | '.' | '!' => format!("\\{}", c),
                        _ => c.to_string(),
                    })
                    .collect()
            }

            let full_message = format!(
                "❁❀❁❀ 🌅 *Remembrance* 🕌 ❀❁❀❁\n\
                ━━━━━━━━━━━━━━━━━━━━━\n\
                *{}* \n\n\
                ✨ *𝒜𝓇𝒶𝒷𝒾𝒸 𝒯𝑒𝓍𝓉:*\n\
                `{}`\n\n\
                🌟  *𝒯𝓇𝒶𝓃𝓈𝓁𝒾𝓉𝑒𝓇𝒶𝓉𝒾𝑜𝓃:*\n\
                `{}`\n\n\
                🔤 *𝒯𝓇𝒶𝓃𝓈𝓁𝒶𝓉𝒾𝑜𝓃:*\n\
                `{}`\n\n\
                📚 *𝑅𝑒𝒻𝑒𝓇𝑒𝓃𝒸𝑒:*\n\
                `{}`\n\n\
                ━━━━━━━━━━━━━━━━━━━━━",
                 &template.message,  
                 &template.arabic, 
                 escape_markdown_v2(&template.transliteration),  
                 escape_markdown_v2(&template.translation), 
                 escape_markdown_v2(&template.reference)
            );
            
            if let Err(e) = bot
                .send_message(ChatId(*user_id), &full_message)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await
            {
                log::error!("Failed to send reminder to user {}: {}", user_id, e);
                continue;
            }
        }
    }

    drop(preferences);

    let mut preferences = match state.acquire_preferences_lock().await {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to acquire lock for updating reminder times: {}", e);
            return;
        }
    };

    for (_, prefs) in preferences.iter_mut() {
        if prefs.opted_in {
            prefs.last_reminder = Some(now);
        }
    }

    drop(preferences);

    if let Err(_) = tokio::time::timeout(Duration::from_secs(5), state.save_preferences()).await {
        log::error!("Timeout while saving preferences in reminder sender");
    }
}
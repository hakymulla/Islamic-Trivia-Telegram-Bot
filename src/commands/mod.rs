use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]

pub enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Get a random trivia question")]
    Question,
    #[command(description = "Show leaderboard")]
    Leaderboard,
    #[command(description = "Start a themed quiz")]
    Theme(String),
    #[command(description = "Opt in to receive reminders")]
    OptIn,
    #[command(description = "Opt out of reminders")]
    OptOut,
    #[command(description = "Show your reminder preferences")]
    Preferences,
    #[command(description = "Show help message")]
    Help,
   
}

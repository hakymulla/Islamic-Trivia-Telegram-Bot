use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::types::Question;

pub fn create_keyboard(
    question: &Question,
    selected_answer: Option<&str>,
    show_correct: bool,
    show_end_button: bool,
) -> InlineKeyboardMarkup {
    let options = question.get_options();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = options
        .iter()
        .map(|option| {
            let mut text = option.clone();
            if let Some(selected) = selected_answer {
                if option == selected {
                    if show_correct && option == &question.correct_answer {
                        text = format!("✅ {}", option);
                    } else if show_correct {
                        text = format!("❌ {}", option);
                    }
                } else if show_correct && option == &question.correct_answer {
                    text = format!("✅ {}", option);
                }
            }
            vec![InlineKeyboardButton::callback(text, option.clone())]
        })
        .collect();

    if show_end_button {
        keyboard.push(vec![InlineKeyboardButton::callback("🛑 End Quiz".to_string(), "end_quiz".to_string())]);
    }
    
    InlineKeyboardMarkup::new(keyboard)
}

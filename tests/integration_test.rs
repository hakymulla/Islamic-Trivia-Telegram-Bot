#[cfg(test)]
mod tests {
    use islamic_trivia_quiz_bot::*;
    use std::error::Error;
    use teloxide::types::InlineKeyboardMarkup;
    // use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use std::collections::HashMap;
    use tokio::sync::Mutex;
    use std::sync::Arc;
    use chrono::{Utc, DateTime};
    use rand::{SeedableRng, rngs::StdRng, seq::IteratorRandom};


    // Helper function to create a test question
    fn create_test_question() -> Question {
        Question {
            id: 1,
            question: String::from("What is the capital of France?"),
            correct_answer: String::from("Paris"),
            option1: String::from("Paris"),
            option2: String::from("London"),
            option3: String::from("Berlin"),
            option4: String::from("Madrid"),
            category: String::from("Geography"),
            points: 10,
        }
    }

    #[warn(dead_code)]
    // Helper function to create a temporary CSV file with test questions
    fn create_test_csv() -> Result<NamedTempFile, Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        
        writeln!(
            temp_file,
            "id,question,correct_answer,option1,option2,option3,option4,category,points"
        )?;
        writeln!(
            temp_file,
            "1,What is the capital of France?,Paris,Paris,London,Berlin,Madrid,Geography,10"
        )?;
        writeln!(
            temp_file,
            "2,Which planet is closest to the Sun?,Mercury,Mercury,Venus,Earth,Mars,Science,10"
        )?;
        
        Ok(temp_file)
    }

    
    // Test Question struct methods
    #[test]
    fn test_question_get_options() {
        let question = create_test_question();
        let options = question.get_options();
        
        assert_eq!(options.len(), 4);
        assert_eq!(options[0], "Paris");
        assert_eq!(options[1], "London");
        assert_eq!(options[2], "Berlin");
        assert_eq!(options[3], "Madrid");
    }

    // Test keyboard creation
    #[test]
    fn test_create_keyboard_initial() {
        let question = create_test_question();
        let keyboard = create_keyboard(&question, None, false, true);
        
        // Check that keyboard has correct number of buttons
        if let InlineKeyboardMarkup { inline_keyboard } = keyboard {
            assert_eq!(inline_keyboard.len(), 5); // 4 options + End Button

            let button_text = &inline_keyboard[4][0].text;
            assert!(button_text.contains("🛑"));
            
            for row in inline_keyboard {
                eprintln!("row: {:?}", row);
                assert_eq!(row.len(), 1);
                let button_text = &row[0].text;
                assert!(!button_text.contains("✅"));
                assert!(!button_text.contains("❌"));
            }
        }
    }

    #[test]
    fn test_create_keyboard_with_correct_answer() {
        let question = create_test_question();
        let keyboard = create_keyboard(&question, Some("Paris"), true, true);
        
        if let InlineKeyboardMarkup { inline_keyboard } = keyboard {
            // Find the button with the selected answer
            let correct_button = inline_keyboard.iter()
                .find(|row| row[0].text.contains("Paris"))
                .unwrap();
            
            // Check that correct answer has green checkmark
            assert!(correct_button[0].text.starts_with("✅"));
        }
    }

    #[test]
    fn test_create_keyboard_with_incorrect_answer() {
        let question = create_test_question();
        let keyboard = create_keyboard(&question, Some("London"), true, true);
        
        if let InlineKeyboardMarkup { inline_keyboard } = keyboard {
            // Find the button with the selected wrong answer
            let incorrect_button = inline_keyboard.iter()
                .find(|row| row[0].text.contains("London"))
                .unwrap();
            
            // Find the button with the correct answer
            let correct_button = inline_keyboard.iter()
                .find(|row| row[0].text.contains("Paris"))
                .unwrap();
            
            // Check that wrong answer has red X and correct answer has green checkmark
            assert!(incorrect_button[0].text.starts_with("❌"));
            assert!(correct_button[0].text.starts_with("✅"));
        }
    }

    // Test CSV loading
    #[test]
    fn test_load_questions() -> Result<(), Box<dyn Error>> {
        let questions = load_questions()?;
        
        assert_eq!(questions.len(), 5);
        assert_eq!(questions[0].question, "What is Rust's ownership model?");
        assert_eq!(questions[1].question, "Which planet is known as the Red Planet?");
        
        Ok(())
    }

    // Test BotState initialization
    #[test]
    fn test_bot_state_initialization() {
        let questions = vec![load_questions()][0].as_ref().unwrap().to_vec();
        let state = Arc::new(BotState {
            questions,
            active_questions: Mutex::new(HashMap::new()),
            user_scores: Mutex::new(HashMap::new()),
            rng: Mutex::new(StdRng::from_entropy()),
        });
        
        assert_eq!(state.questions.len(), 5);
    }

    // Test score tracking
    #[tokio::test]
    async fn test_user_score_tracking() {
        let state = Arc::new(BotState {
            questions: vec![create_test_question()],
            active_questions: Mutex::new(HashMap::new()),
            user_scores: Mutex::new(HashMap::new()),
            rng: Mutex::new(StdRng::from_entropy()),
        });

        let user_id = 12345i64;
        let username = String::from("TestUser");

        // Create a test score
        let mut scores = state.user_scores.lock().await;
        scores.insert(user_id, UserScore {
            user_id,
            username: username.clone(),
            score: 10,
            last_answer_time: Utc::now(),
        });

        // Check if score was properly recorded
        let user_score = scores.get(&user_id).unwrap();
        assert_eq!(user_score.score, 10);
        assert_eq!(user_score.username, username);
    }

    // // Test theme filtering
    // #[test]
    // fn test_theme_filtering() {
    //     let questions = vec![
    //         Question {
    //             id: 1,
    //             question: String::from("What is the capital of France?"),
    //             correct_answer: String::from("Paris"),
    //             option1: String::from("Paris"),
    //             option2: String::from("London"),
    //             option3: String::from("Berlin"),
    //             option4: String::from("Madrid"),
    //             category: String::from("Geography"),
    //             points: 10,
    //         },
    //         Question {
    //             id: 2,
    //             question: String::from("What is 2+2?"),
    //             correct_answer: String::from("4"),
    //             option1: String::from("3"),
    //             option2: String::from("4"),
    //             option3: String::from("5"),
    //             option4: String::from("6"),
    //             category: String::from("Math"),
    //             points: 10,
    //         },
    //     ];

    //     let filtered = questions.iter()
    //         .filter(|q| q.category.to_lowercase() == "geography")
    //         .collect::<Vec<_>>();

    //     assert_eq!(filtered.len(), 1);
    //     assert_eq!(filtered[0].category, "Geography");
    // }

    // // Integration test simulating a question-answer flow
    // #[tokio::test]
    // async fn test_question_answer_flow() {
    //     let state = Arc::new(BotState {
    //         questions: vec![create_test_question()],
    //         active_questions: Mutex::new(HashMap::new()),
    //         user_scores: Mutex::new(HashMap::new()),
    //         rng: Mutex::new(StdRng::from_entropy()),
    //     });

    //     let chat_id = 12345i64;
    //     let message_id = MessageId(1);

    //     // Simulate sending a question
    //     let question = state.questions[0].clone();
    //     state.active_questions.lock().await.insert(
    //         chat_id,
    //         ActiveQuestion {
    //             question: question.clone(),
    //             message_id,
    //         },
    //     );

    //     // Verify question is active
    //     let active_questions = state.active_questions.lock().await;
    //     assert!(active_questions.contains_key(&chat_id));

    //     // Simulate correct answer
    //     let active_question = active_questions.get(&chat_id).unwrap();
    //     assert_eq!(active_question.question.correct_answer, "Paris");
    // }
}
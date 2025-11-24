//! Sorting session module for the Discord Sorting Hat bot.
//!
//! This module handles the sorting quiz logic, including tracking user progress,
//! recording answers, and calculating the final house assignment based on trait scores.

use crate::config::{Config, House};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Session timeout duration in seconds (30 minutes).
const SESSION_TIMEOUT_SECS: u64 = 30 * 60;

/// Represents an active sorting session for a user.
///
/// Tracks the user's progress through the sorting quiz, including their answers
/// and when the session was created for expiration purposes.
pub struct SortingSession {
    /// Index of the current question (0-based).
    pub current_question: usize,
    /// List of answer indices selected by the user for each question.
    pub answers: Vec<usize>,
    /// Timestamp when the session was created, used for expiration checks.
    pub created_at: Instant,
}

impl SortingSession {
    /// Creates a new sorting session for a user.
    ///
    /// # Arguments
    ///
    /// * `_config` - Reference to the bot configuration (reserved for future use).
    ///
    /// # Returns
    ///
    /// A new `SortingSession` initialized at the first question with no answers.
    pub fn new(_config: Arc<Config>) -> Self {
        Self {
            current_question: 0,
            answers: Vec::new(),
            created_at: Instant::now(),
        }
    }

    /// Checks if the session has expired.
    ///
    /// Sessions expire after 30 minutes of inactivity to prevent stale sessions
    /// from accumulating in memory.
    ///
    /// # Returns
    ///
    /// `true` if the session has exceeded the timeout duration, `false` otherwise.
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs() > SESSION_TIMEOUT_SECS
    }

    /// Records a user's answer for the current question.
    ///
    /// Stores the selected answer index and advances to the next question.
    ///
    /// # Arguments
    ///
    /// * `answer_index` - The 0-based index of the selected answer option.
    pub fn record_answer(&mut self, answer_index: usize) {
        self.answers.push(answer_index);
        self.current_question += 1;
    }

    /// Determines which house the user should be sorted into.
    ///
    /// Calculates trait scores based on all recorded answers, then matches
    /// those scores against each house's associated traits to find the best fit.
    ///
    /// # Arguments
    ///
    /// * `config` - Reference to the bot configuration containing houses and questions.
    ///
    /// # Returns
    ///
    /// A clone of the `House` with the highest matching score.
    ///
    /// # Algorithm
    ///
    /// 1. Accumulate trait points from all selected answers
    /// 2. For each house, sum the points of its associated traits
    /// 3. Return the house with the highest total score
    pub fn determine_house(&self, config: &Config) -> House {
        // Calculate trait scores based on answers
        let mut trait_scores: HashMap<String, u32> = HashMap::new();

        for (question_idx, &answer_idx) in self.answers.iter().enumerate() {
            if let Some(question) = config.questions.get(question_idx) {
                if let Some(option) = question.options.get(answer_idx) {
                    for (trait_name, &score) in &option.traits {
                        *trait_scores.entry(trait_name.clone()).or_insert(0) += score;
                    }
                }
            }
        }

        // Calculate house scores based on trait matches
        let mut house_scores: Vec<(usize, u32)> = config
            .houses
            .iter()
            .enumerate()
            .map(|(idx, house)| {
                let score: u32 = house
                    .traits
                    .iter()
                    .filter_map(|trait_name| trait_scores.get(trait_name))
                    .sum();
                (idx, score)
            })
            .collect();

        // Sort by score (descending)
        house_scores.sort_by(|a, b| b.1.cmp(&a.1));

        // Return the house with the highest score
        let best_house_idx = house_scores[0].0;
        config.houses[best_house_idx].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{House, Question, QuestionOption};

    /// Tests the basic sorting session functionality.
    ///
    /// Verifies that a user who selects a "brave" option is sorted into
    /// the house associated with the "brave" trait.
    #[test]
    fn test_sorting_session() {
        let houses = vec![
            House {
                name: "Gryffindor".to_string(),
                role_name: "Gryffindor".to_string(),
                description: "Brave and daring".to_string(),
                traits: vec!["brave".to_string(), "daring".to_string()],
            },
            House {
                name: "Ravenclaw".to_string(),
                role_name: "Ravenclaw".to_string(),
                description: "Intelligent and wise".to_string(),
                traits: vec!["intelligent".to_string(), "wise".to_string()],
            },
        ];

        let questions = vec![Question {
            question: "Test question?".to_string(),
            options: vec![
                QuestionOption {
                    text: "Brave option".to_string(),
                    traits: HashMap::from([("brave".to_string(), 3)]),
                },
                QuestionOption {
                    text: "Smart option".to_string(),
                    traits: HashMap::from([("intelligent".to_string(), 3)]),
                },
            ],
        }];

        let config = Arc::new(Config { houses, questions });
        let mut session = SortingSession::new(config.clone());

        // Choose brave option
        session.record_answer(0);

        let house = session.determine_house(&config);
        assert_eq!(house.name, "Gryffindor");
    }
}

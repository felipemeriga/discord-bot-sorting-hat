use crate::config::{Config, House};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

pub struct SortingSession {
    pub current_question: usize,
    pub answers: Vec<usize>,
    pub created_at: Instant,
}

impl SortingSession {
    pub fn new(_config: Arc<Config>) -> Self {
        Self {
            current_question: 0,
            answers: Vec::new(),
            created_at: Instant::now(),
        }
    }

    /// Check if session has expired (default: 30 minutes)
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs() > 30 * 60
    }

    pub fn record_answer(&mut self, answer_index: usize) {
        self.answers.push(answer_index);
        self.current_question += 1;
    }

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

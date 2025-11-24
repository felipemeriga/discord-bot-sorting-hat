//! Configuration module for the Discord Sorting Hat bot.
//!
//! This module defines the data structures used to configure the bot's behavior,
//! including houses, questions, and trait mappings loaded from config.json.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration structure containing all bot settings.
///
/// This struct is deserialized from the config.json file and contains
/// the complete configuration for houses and sorting questions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// List of available houses that members can be sorted into.
    pub houses: Vec<House>,
    /// List of questions used in the sorting quiz.
    pub questions: Vec<Question>,
}

/// Represents a house that members can be sorted into.
///
/// Each house has associated traits that are matched against user answers
/// to determine the best fit during the sorting process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct House {
    /// Display name of the house (e.g., "Draco", "Lupus").
    pub name: String,
    /// The exact role name in Discord that will be assigned to sorted members.
    pub role_name: String,
    /// Description shown to the user when they are sorted into this house (in Portuguese).
    pub description: String,
    /// List of trait names associated with this house for scoring purposes.
    pub traits: Vec<String>,
}

/// Represents a single question in the sorting quiz.
///
/// Questions are presented to users in Portuguese and each has multiple
/// answer options with associated trait scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    /// The question text displayed to the user (in Portuguese).
    pub question: String,
    /// Available answer options for this question.
    pub options: Vec<QuestionOption>,
}

/// Represents an answer option for a sorting question.
///
/// Each option has associated traits with point values that contribute
/// to the final house determination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    /// The answer text displayed to the user (in Portuguese).
    pub text: String,
    /// Map of trait names to point values awarded when this option is selected.
    pub traits: HashMap<String, u32>,
}

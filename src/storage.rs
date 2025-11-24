//! Storage module for persisting sorted users.
//!
//! This module provides functionality to persist sorted users to a JSON file,
//! allowing the bot to remember which users have been sorted even after restarts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Default filename for storing sorted users data.
const STORAGE_FILE: &str = "sorted_users.json";

/// Represents a user who has been sorted into a house.
///
/// Contains the house assignment and timestamp of when the sorting occurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortedUser {
    /// The name of the house the user was sorted into.
    pub house_name: String,
    /// ISO 8601 timestamp of when the user was sorted.
    pub sorted_at: String,
}

/// Thread-safe storage for sorted users with file persistence.
///
/// Provides methods to load, save, check, and add sorted users.
/// Uses a `RwLock` for thread-safe concurrent access.
pub struct SortedUsersStorage {
    /// Map of user IDs (as strings) to their sorted user data.
    users: Arc<RwLock<HashMap<String, SortedUser>>>,
    /// Path to the storage file.
    file_path: String,
}

impl SortedUsersStorage {
    /// Creates a new storage instance and loads existing data from file.
    ///
    /// If the storage file doesn't exist, starts with an empty map.
    /// If the file exists but is invalid JSON, logs a warning and starts fresh.
    ///
    /// # Returns
    ///
    /// A new `SortedUsersStorage` instance with loaded data.
    pub fn new() -> Self {
        let file_path = STORAGE_FILE.to_string();
        let users = Self::load_from_file(&file_path);

        Self {
            users: Arc::new(RwLock::new(users)),
            file_path,
        }
    }

    /// Loads sorted users data from the storage file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the JSON storage file.
    ///
    /// # Returns
    ///
    /// A `HashMap` of user IDs to `SortedUser` data. Returns empty map if file
    /// doesn't exist or contains invalid JSON.
    fn load_from_file(file_path: &str) -> HashMap<String, SortedUser> {
        if !Path::new(file_path).exists() {
            tracing::info!("Storage file not found, starting with empty sorted users list");
            return HashMap::new();
        }

        match fs::read_to_string(file_path) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(users) => {
                        tracing::info!("Loaded sorted users from {}", file_path);
                        users
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse storage file: {:?}, starting fresh", e);
                        HashMap::new()
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read storage file: {:?}, starting fresh", e);
                HashMap::new()
            }
        }
    }

    /// Saves the current sorted users data to the storage file.
    ///
    /// # Returns
    ///
    /// `Ok(())` if save was successful, `Err` with error message otherwise.
    async fn save_to_file(&self) -> Result<(), String> {
        let users = self.users.read().await;
        let json = serde_json::to_string_pretty(&*users)
            .map_err(|e| format!("Failed to serialize users: {:?}", e))?;

        fs::write(&self.file_path, json)
            .map_err(|e| format!("Failed to write storage file: {:?}", e))?;

        tracing::debug!("Saved sorted users to {}", self.file_path);
        Ok(())
    }

    /// Checks if a user has already been sorted.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The Discord user ID to check.
    ///
    /// # Returns
    ///
    /// `Some(SortedUser)` if the user has been sorted, `None` otherwise.
    pub async fn get_sorted_user(&self, user_id: u64) -> Option<SortedUser> {
        let users = self.users.read().await;
        users.get(&user_id.to_string()).cloned()
    }

    /// Checks if a user has already been sorted.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The Discord user ID to check.
    ///
    /// # Returns
    ///
    /// `true` if the user has been sorted, `false` otherwise.
    pub async fn is_sorted(&self, user_id: u64) -> bool {
        let users = self.users.read().await;
        users.contains_key(&user_id.to_string())
    }

    /// Adds a user to the sorted users list and persists to file.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The Discord user ID.
    /// * `house_name` - The name of the house the user was sorted into.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the user was added and saved successfully, `Err` otherwise.
    pub async fn add_sorted_user(&self, user_id: u64, house_name: &str) -> Result<(), String> {
        let sorted_user = SortedUser {
            house_name: house_name.to_string(),
            sorted_at: chrono::Utc::now().to_rfc3339(),
        };

        {
            let mut users = self.users.write().await;
            users.insert(user_id.to_string(), sorted_user);
        }

        self.save_to_file().await
    }

    /// Returns the total number of sorted users.
    ///
    /// # Returns
    ///
    /// The count of users who have been sorted.
    pub async fn count(&self) -> usize {
        let users = self.users.read().await;
        users.len()
    }
}

impl Default for SortedUsersStorage {
    fn default() -> Self {
        Self::new()
    }
}

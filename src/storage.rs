//! Storage module for persisting sorted users.
//!
//! This module provides configurable storage backends for persisting sorted users.
//! Supports both local JSON file storage (for development) and AWS S3 storage (for production).
//!
//! Configure via environment variables:
//! - `STORAGE_TYPE=local` (default) - Uses local JSON file
//! - `STORAGE_TYPE=s3` - Uses AWS S3 bucket
//!
//! For S3 storage, also set:
//! - `S3_BUCKET_NAME` - The S3 bucket name
//! - `S3_REGION` - AWS region (optional, defaults to us-east-1)
//! - `S3_STORAGE_KEY` - S3 object key (optional, defaults to sorted_users.json)

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client as S3Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Default filename for local storage.
const LOCAL_STORAGE_FILE: &str = "sorted_users.json";

/// Default S3 object key.
const DEFAULT_S3_KEY: &str = "sorted_users.json";

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

/// Trait for storage backend implementations.
///
/// Provides async methods for loading, saving, and querying sorted users.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Loads all sorted users from storage.
    async fn load(&self) -> Result<HashMap<String, SortedUser>, String>;

    /// Saves all sorted users to storage.
    async fn save(&self, users: &HashMap<String, SortedUser>) -> Result<(), String>;
}

/// Local JSON file storage backend.
///
/// Stores sorted users in a local JSON file on the filesystem.
pub struct LocalStorage {
    file_path: String,
}

impl LocalStorage {
    /// Creates a new local storage backend.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the JSON storage file.
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn load(&self) -> Result<HashMap<String, SortedUser>, String> {
        if !Path::new(&self.file_path).exists() {
            tracing::info!("Local storage file not found, starting with empty list");
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read local storage: {:?}", e))?;

        let users: HashMap<String, SortedUser> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse local storage: {:?}", e))?;

        tracing::info!("Loaded {} users from local storage", users.len());
        Ok(users)
    }

    async fn save(&self, users: &HashMap<String, SortedUser>) -> Result<(), String> {
        let json = serde_json::to_string_pretty(users)
            .map_err(|e| format!("Failed to serialize users: {:?}", e))?;

        fs::write(&self.file_path, json)
            .map_err(|e| format!("Failed to write local storage: {:?}", e))?;

        tracing::debug!("Saved {} users to local storage", users.len());
        Ok(())
    }
}

/// AWS S3 storage backend.
///
/// Stores sorted users in an S3 object for durable cloud storage.
pub struct S3Storage {
    client: S3Client,
    bucket_name: String,
    object_key: String,
}

impl S3Storage {
    /// Creates a new S3 storage backend.
    ///
    /// # Arguments
    ///
    /// * `bucket_name` - The S3 bucket name.
    /// * `object_key` - The S3 object key (file name in bucket).
    pub async fn new(bucket_name: String, object_key: String) -> Result<Self, String> {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = S3Client::new(&config);

        tracing::info!(
            "Initialized S3 storage: bucket={}, key={}",
            bucket_name,
            object_key
        );

        Ok(Self {
            client,
            bucket_name,
            object_key,
        })
    }
}

#[async_trait]
impl StorageBackend for S3Storage {
    async fn load(&self) -> Result<HashMap<String, SortedUser>, String> {
        match self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(&self.object_key)
            .send()
            .await
        {
            Ok(output) => {
                let bytes = output
                    .body
                    .collect()
                    .await
                    .map_err(|e| format!("Failed to read S3 object body: {:?}", e))?
                    .into_bytes();

                let content = String::from_utf8(bytes.to_vec())
                    .map_err(|e| format!("S3 object is not valid UTF-8: {:?}", e))?;

                let users: HashMap<String, SortedUser> = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse S3 storage: {:?}", e))?;

                tracing::info!("Loaded {} users from S3 storage", users.len());
                Ok(users)
            }
            Err(e) => {
                // If object doesn't exist, start with empty map
                if e.to_string().contains("NoSuchKey") {
                    tracing::info!("S3 object not found, starting with empty list");
                    Ok(HashMap::new())
                } else {
                    Err(format!("Failed to load from S3: {:?}", e))
                }
            }
        }
    }

    async fn save(&self, users: &HashMap<String, SortedUser>) -> Result<(), String> {
        let json = serde_json::to_string_pretty(users)
            .map_err(|e| format!("Failed to serialize users: {:?}", e))?;

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&self.object_key)
            .body(json.into_bytes().into())
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to save to S3: {:?}", e))?;

        tracing::debug!("Saved {} users to S3 storage", users.len());
        Ok(())
    }
}

/// Thread-safe storage for sorted users with pluggable backends.
///
/// Provides methods to load, save, check, and add sorted users.
/// Uses a `RwLock` for thread-safe concurrent access.
pub struct SortedUsersStorage {
    /// Map of user IDs (as strings) to their sorted user data.
    users: Arc<RwLock<HashMap<String, SortedUser>>>,
    /// The storage backend implementation.
    backend: Arc<dyn StorageBackend>,
}

impl SortedUsersStorage {
    /// Creates a new storage instance with the specified backend.
    ///
    /// Loads existing data from the backend on initialization.
    ///
    /// # Arguments
    ///
    /// * `backend` - The storage backend to use.
    ///
    /// # Returns
    ///
    /// A new `SortedUsersStorage` instance with loaded data.
    pub async fn new(backend: Arc<dyn StorageBackend>) -> Result<Self, String> {
        let users = backend.load().await.unwrap_or_else(|e| {
            tracing::warn!("Failed to load users from storage: {}, starting fresh", e);
            HashMap::new()
        });

        Ok(Self {
            users: Arc::new(RwLock::new(users)),
            backend,
        })
    }

    /// Creates a storage instance based on environment configuration.
    ///
    /// Reads `STORAGE_TYPE` environment variable to determine backend:
    /// - `local` (default): Uses local JSON file storage
    /// - `s3`: Uses AWS S3 storage (requires S3_BUCKET_NAME)
    ///
    /// # Returns
    ///
    /// A configured `SortedUsersStorage` instance.
    pub async fn from_env() -> Result<Self, String> {
        let storage_type = env::var("STORAGE_TYPE").unwrap_or_else(|_| "local".to_string());

        let backend: Arc<dyn StorageBackend> = match storage_type.to_lowercase().as_str() {
            "s3" => {
                let bucket_name = env::var("S3_BUCKET_NAME")
                    .map_err(|_| "S3_BUCKET_NAME must be set when using S3 storage".to_string())?;

                let object_key = env::var("S3_STORAGE_KEY")
                    .unwrap_or_else(|_| DEFAULT_S3_KEY.to_string());

                let s3_storage = S3Storage::new(bucket_name, object_key).await?;
                Arc::new(s3_storage)
            }
            "local" | _ => {
                let file_path = LOCAL_STORAGE_FILE.to_string();
                tracing::info!("Using local storage: {}", file_path);
                Arc::new(LocalStorage::new(file_path))
            }
        };

        Self::new(backend).await
    }

    /// Saves the current sorted users data to the storage backend.
    ///
    /// # Returns
    ///
    /// `Ok(())` if save was successful, `Err` with error message otherwise.
    async fn save_to_backend(&self) -> Result<(), String> {
        let users = self.users.read().await;
        self.backend.save(&*users).await
    }

    /// Checks if a user has already been sorted and returns their data.
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

    /// Adds a user to the sorted users list and persists to storage.
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

        self.save_to_backend().await
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

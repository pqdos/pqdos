//! GitHub Storage Backend for pqdos
//!
//! This module provides a client for storing and retrieving blocks and blockchains
//! from a GitHub repository. Blocks are stored as JSON files with their SHA-256 hash
//! as the filename.

use base64::{engine::general_purpose, Engine as _};
use chrono;
use reqwest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use super::traits::{StorageConfig, StorageError, StorageResult, StoredBlock, StoredBlockchain};

/// Default GitHub API base URL
const GITHUB_API_URL: &str = "https://api.github.com";

/// Default repository name for pqdos blockchain
const DEFAULT_REPO: &str = "blockchain";

/// Default owner for pqdos blockchain
const DEFAULT_OWNER: &str = "pqdos";

/// GitHub Storage configuration
#[derive(Debug, Clone)]
pub struct GitHubConfig {
    pub api_url: String,
    pub owner: String,
    pub repo: String,
    pub token: Option<String>,
    pub user_agent: String,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            api_url: GITHUB_API_URL.to_string(),
            owner: DEFAULT_OWNER.to_string(),
            repo: DEFAULT_REPO.to_string(),
            token: None,
            user_agent: "pqdos-storage".to_string(),
        }
    }
}

impl GitHubConfig {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            api_url: GITHUB_API_URL.to_string(),
            owner: owner.into(),
            repo: repo.into(),
            token: None,
            user_agent: "pqdos-storage".to_string(),
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn from_storage_config(config: &StorageConfig) -> Self {
        let mut cfg = Self::default();
        if let Some(owner) = config.parameters.get("owner") {
            cfg.owner = owner.clone();
        }
        if let Some(repo) = config.parameters.get("repo") {
            cfg.repo = repo.clone();
        }
        if let Some(api_url) = config.parameters.get("api_url") {
            cfg.api_url = api_url.clone();
        }
        if let Some(token) = config.parameters.get("token") {
            cfg.token = Some(token.clone());
        }
        if let Some(user_agent) = config.parameters.get("user_agent") {
            cfg.user_agent = user_agent.clone();
        }
        cfg
    }
}

/// GitHub Storage Backend
#[derive(Debug, Clone)]
pub struct GitHubStorage {
    pub config: GitHubConfig,
    pub client: Arc<reqwest::Client>,
    pub storage_id: String,
    pub owner_id: String,
}

impl GitHubStorage {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        let owner_str: String = owner.into();
        let repo_str: String = repo.into();
        let config = GitHubConfig::new(owner_str.clone(), repo_str.clone());
        Self {
            config,
            client: Arc::new(reqwest::Client::new()),
            storage_id: format!("{}/{}", owner_str, repo_str),
            owner_id: owner_str,
        }
    }

    pub fn with_config(config: GitHubConfig, owner_id: impl Into<String>) -> Self {
        let storage_id = format!("{}/{}", config.owner, config.repo);
        Self {
            config,
            client: Arc::new(reqwest::Client::new()),
            storage_id,
            owner_id: owner_id.into(),
        }
    }

    pub fn from_config(config: &StorageConfig) -> Self {
        let github_config = GitHubConfig::from_storage_config(config);
        Self::with_config(github_config, config.owner_id.clone())
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.config.token = Some(token.into());
        self
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}/repos/{}/{}/{}", self.config.api_url, self.config.owner, self.config.repo, path)
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Accept",
            reqwest::header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );
        headers.insert(
            "User-Agent",
            reqwest::header::HeaderValue::from_str(&self.config.user_agent).unwrap(),
        );
        if let Some(token) = &self.config.token {
            headers.insert(
                "Authorization",
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            );
        }
        headers
    }

    pub async fn fetch_file(&self, path: &str) -> StorageResult<Option<Vec<u8>>> {
        let url = self.api_url(path);
        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        match response.status() {
            | reqwest::StatusCode::OK => {
                let json: serde_json::Value = response
                    .json()
                    .await
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
                let content = json["content"].as_str().ok_or_else(|| {
                    StorageError::InternalError("Invalid response format".to_string())
                })?;
                Ok(Some(
                    general_purpose::STANDARD
                        .decode(content)
                        .map_err(|e| StorageError::DeserializationError(e.to_string()))?,
                ))
            },
            | reqwest::StatusCode::NOT_FOUND => Ok(None),
            | reqwest::StatusCode::UNAUTHORIZED => Err(StorageError::PermissionDenied),
            | _ => {
                Err(StorageError::ConnectionError(format!("API error: {:?}", response.status())))
            },
        }
    }

    pub async fn store_file(&self, path: &str, content: &[u8]) -> StorageResult<()> {
        let _ = self.config.token.as_ref().ok_or(StorageError::PermissionDenied)?;
        let url = self.api_url(path);
        let sha = match self.get_file_sha(path).await {
            | Ok(sha) => Some(sha),
            | Err(_) => None,
        };
        let request_body = serde_json::json!({
            "message": "pqdos store",
            "content": general_purpose::STANDARD.encode(content),
            "sha": sha
        });
        let response = self
            .client
            .put(&url)
            .headers(self.headers())
            .json(&request_body)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        if !response.status().is_success() {
            return Err(StorageError::ConnectionError(format!(
                "API error: {:?}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn list_files(&self, path: &str) -> StorageResult<Vec<String>> {
        let url = self.api_url(path);
        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }
        let json: serde_json::Value =
            response.json().await.map_err(|e| StorageError::DeserializationError(e.to_string()))?;
        Ok(json
            .as_array()
            .map(|a| a.iter().filter_map(|v| v["name"].as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default())
    }

    pub async fn get_file_sha(&self, path: &str) -> StorageResult<String> {
        let url = self.api_url(path);
        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        if response.status() != reqwest::StatusCode::OK {
            return Err(StorageError::NotFound);
        }
        let json: serde_json::Value =
            response.json().await.map_err(|e| StorageError::DeserializationError(e.to_string()))?;
        Ok(json["sha"]
            .as_str()
            .ok_or_else(|| StorageError::InternalError("Missing sha in response".to_string()))?
            .to_string())
    }

    pub async fn delete_file(&self, path: &str) -> StorageResult<()> {
        let _ = self.config.token.as_ref().ok_or(StorageError::PermissionDenied)?;
        let url = self.api_url(path);
        let sha = self.get_file_sha(path).await?;
        let response = self
            .client
            .delete(&url)
            .headers(self.headers())
            .json(&serde_json::json!({"message": "delete", "sha": sha}))
            .send()
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        if !response.status().is_success() {
            return Err(StorageError::ConnectionError(format!(
                "API error: {:?}",
                response.status()
            )));
        }
        Ok(())
    }
}

// ============================================================================
// STORED BLOCK IMPLEMENTATION
// ============================================================================

/// Block stored in GitHub repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubBlock {
    pub id: String,
    #[serde(rename = "previous_id")]
    pub previous_id: Option<String>,
    pub data: String,
    #[serde(rename = "owner_id")]
    pub owner_id: String,
    #[serde(rename = "type")]
    pub block_type: String,
    pub timestamp: i64,
    pub signature: Option<String>,
    pub signer: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl GitHubBlock {
    pub fn new(
        id: impl Into<String>,
        data: impl AsRef<[u8]>,
        owner_id: impl Into<String>,
        block_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            previous_id: None,
            data: general_purpose::STANDARD.encode(data.as_ref()),
            owner_id: owner_id.into(),
            block_type: block_type.into(),
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
            signer: None,
            metadata: HashMap::new(),
        }
    }

    pub fn new_with_previous(
        id: impl Into<String>,
        data: impl AsRef<[u8]>,
        owner_id: impl Into<String>,
        block_type: impl Into<String>,
        previous_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            previous_id: Some(previous_id.into()),
            data: general_purpose::STANDARD.encode(data.as_ref()),
            owner_id: owner_id.into(),
            block_type: block_type.into(),
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
            signer: None,
            metadata: HashMap::new(),
        }
    }

    pub fn compute_id(data: &[u8]) -> String {
        format!("{:x}", Sha256::digest(data))
    }

    pub fn raw_data(&self) -> StorageResult<Vec<u8>> {
        general_purpose::STANDARD
            .decode(&self.data)
            .map_err(|e| StorageError::DeserializationError(format!("Base64 decode error: {}", e)))
    }
}

impl StoredBlock for GitHubBlock {
    fn id(&self) -> &str {
        &self.id
    }
    fn previous_id(&self) -> Option<&str> {
        self.previous_id.as_deref()
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn owner_id(&self) -> &str {
        &self.owner_id
    }
    fn block_type(&self) -> &str {
        &self.block_type
    }
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
    fn signature(&self) -> Option<&str> {
        self.signature.as_deref()
    }
    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

// ============================================================================
// STORED BLOCKCHAIN IMPLEMENTATION
// ============================================================================

/// Blockchain stored in GitHub repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubBlockchain {
    pub name: String,
    pub genesis_block: String,
    pub head_block: String,
    pub blocks: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub description: Option<String>,
}

impl GitHubBlockchain {
    pub fn new(name: impl Into<String>, genesis: impl Into<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        let genesis = genesis.into();
        Self {
            name: name.into(),
            genesis_block: genesis.clone(),
            head_block: genesis.clone(),
            blocks: vec![genesis],
            created_at: now,
            updated_at: now,
            description: None,
        }
    }

    pub fn with_block(mut self, block: impl Into<String>) -> Self {
        self.blocks.push(block.into());
        self.head_block = self.blocks.last().cloned().unwrap_or_default();
        self.updated_at = chrono::Utc::now().timestamp();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl StoredBlockchain for GitHubBlockchain {
    fn name(&self) -> &str {
        &self.name
    }
    fn genesis_block(&self) -> &str {
        &self.genesis_block
    }
    fn head_block(&self) -> &str {
        &self.head_block
    }
    fn blocks(&self) -> &[String] {
        &self.blocks
    }
    fn created_at(&self) -> i64 {
        self.created_at
    }
    fn updated_at(&self) -> i64 {
        self.updated_at
    }
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

// ============================================================================
// ASYNC STORAGE BACKEND TRAIT AND IMPLEMENTATION
// ============================================================================

#[allow(async_fn_in_trait)]
pub trait AsyncStorageBackend: Clone + Debug + Send + Sync {
    type Block: StoredBlock + Clone + Send + Sync;
    type Blockchain: StoredBlockchain + Clone + Send + Sync;
    type Error: std::error::Error + Send + Sync + 'static;

    fn id(&self) -> &str;
    fn backend_type(&self) -> &str;
    fn owner_id(&self) -> &str;

    async fn store_block(&self, block: &Self::Block) -> Result<(), Self::Error>;
    async fn get_block(&self, block_id: &str) -> Result<Option<Self::Block>, Self::Error>;
    async fn has_block(&self, block_id: &str) -> Result<bool, Self::Error>;
    async fn list_block_ids(&self) -> Result<Vec<String>, Self::Error>;
    async fn list_blocks_by_owner(&self, owner_id: &str) -> Result<Vec<Self::Block>, Self::Error>;
    async fn store_blockchain(&self, chain: &Self::Blockchain) -> Result<(), Self::Error>;
    async fn get_blockchain(&self, name: &str) -> Result<Option<Self::Blockchain>, Self::Error>;
    async fn list_blockchain_names(&self) -> Result<Vec<String>, Self::Error>;
    async fn create_block(
        &self,
        data: &[u8],
        owner_id: &str,
        block_type: &str,
        previous_id: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> Result<Self::Block, Self::Error>;
}

impl AsyncStorageBackend for GitHubStorage {
    type Block = GitHubBlock;
    type Blockchain = GitHubBlockchain;
    type Error = StorageError;

    fn id(&self) -> &str {
        &self.storage_id
    }
    fn backend_type(&self) -> &str {
        "github"
    }
    fn owner_id(&self) -> &str {
        &self.owner_id
    }

    async fn store_block(&self, block: &Self::Block) -> Result<(), Self::Error> {
        let path = format!("blocks/{}.json", block.id);
        let content = serde_json::to_vec(block)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        self.store_file(&path, &content).await?;
        Ok(())
    }

    async fn get_block(&self, block_id: &str) -> Result<Option<Self::Block>, Self::Error> {
        let path = format!("blocks/{}.json", block_id);
        match self.fetch_file(&path).await? {
            | Some(bytes) => {
                let block: GitHubBlock = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
                Ok(Some(block))
            },
            | None => Ok(None),
        }
    }

    async fn has_block(&self, block_id: &str) -> Result<bool, Self::Error> {
        let path = format!("blocks/{}.json", block_id);
        Ok(self.fetch_file(&path).await?.is_some())
    }

    async fn list_block_ids(&self) -> Result<Vec<String>, Self::Error> {
        let files = self.list_files("blocks").await?;
        Ok(files
            .into_iter()
            .filter(|f| f.ends_with(".json"))
            .map(|f| f.trim_end_matches(".json").to_string())
            .collect())
    }

    async fn list_blocks_by_owner(&self, owner_id: &str) -> Result<Vec<Self::Block>, Self::Error> {
        let all_block_ids = self.list_block_ids().await?;
        let mut result = Vec::new();
        for block_id in all_block_ids {
            if let Some(block) = self.get_block(&block_id).await? {
                if block.owner_id == owner_id {
                    result.push(block);
                }
            }
        }
        Ok(result)
    }

    async fn store_blockchain(&self, chain: &Self::Blockchain) -> Result<(), Self::Error> {
        let path = format!("chains/{}.json", chain.name);
        let content = serde_json::to_vec(chain)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        self.store_file(&path, &content).await?;
        Ok(())
    }

    async fn get_blockchain(&self, name: &str) -> Result<Option<Self::Blockchain>, Self::Error> {
        let path = format!("chains/{}.json", name);
        match self.fetch_file(&path).await? {
            | Some(bytes) => {
                let chain: GitHubBlockchain = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
                Ok(Some(chain))
            },
            | None => Ok(None),
        }
    }

    async fn list_blockchain_names(&self) -> Result<Vec<String>, Self::Error> {
        let files = self.list_files("chains").await?;
        Ok(files
            .into_iter()
            .filter(|f| f.ends_with(".json"))
            .map(|f| f.trim_end_matches(".json").to_string())
            .collect())
    }

    async fn create_block(
        &self,
        data: &[u8],
        owner_id: &str,
        block_type: &str,
        previous_id: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> Result<Self::Block, Self::Error> {
        let id = GitHubBlock::compute_id(data);
        let mut block = GitHubBlock::new(id.clone(), data, owner_id, block_type);
        if let Some(prev) = previous_id {
            block.previous_id = Some(prev.to_string());
        }
        block.metadata = metadata;
        self.store_block(&block).await?;
        Ok(block)
    }
}

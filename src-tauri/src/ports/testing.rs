//! In-memory fake implementations of infrastructure ports for unit tests.
//!
//! These fakes let the creative engine core flow be tested without Tauri,
//! a real LLM, or a real vector database.

use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;

use super::{LlmPort, LlmPortRequest, VectorStore};
use crate::{
    config::settings::{LlmProfile, LlmProvider},
    domain::{
        contracts::{MasterSettingContract, RuntimeContract},
        creative_engine::RuntimeContractProvider,
    },
    error::AppError,
    llm::adapter::GenerateResponse,
    vector::{SearchResult, VectorRecord},
};

/// Fake LLM port that returns a fixed response and records every request.
pub struct FakeLlmPort {
    response: Mutex<String>,
    calls: Mutex<Vec<LlmPortRequest>>,
}

impl FakeLlmPort {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: Mutex::new(response.into()),
            calls: Mutex::new(Vec::new()),
        }
    }

    pub fn set_response(&self, response: impl Into<String>) {
        *self.response.lock().unwrap() = response.into();
    }

    pub fn calls(&self) -> Vec<LlmPortRequest> {
        self.calls.lock().unwrap().clone()
    }
}

fn default_llm_profile() -> LlmProfile {
    LlmProfile {
        id: "fake-model".to_string(),
        name: "Fake Model".to_string(),
        description: None,
        provider: LlmProvider::Ollama,
        model_source: Default::default(),
        model: "fake".to_string(),
        api_key: String::new(),
        api_base: None,
        is_local_model: true,
        max_tokens: 1024,
        temperature: 0.3,
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        timeout_seconds: 5,
        is_default: false,
        enabled: true,
        kind: Default::default(),
        capabilities: vec![],
        max_context_length: 8192,
        quality_tier: Default::default(),
        speed_tier: Default::default(),
        cost_per_1k_input: None,
        cost_per_1k_output: None,
        tags: vec![],
        supports_system_prompt: true,
        system_prompt_override: None,
        supports_streaming: true,
        knowledge_cutoff: None,
        reasoning_effort: None,
    }
}

#[async_trait]
impl LlmPort for FakeLlmPort {
    async fn generate(&self, request: LlmPortRequest) -> Result<GenerateResponse, AppError> {
        self.calls.lock().unwrap().push(request);
        Ok(GenerateResponse {
            content: self.response.lock().unwrap().clone(),
            model: "fake".to_string(),
            tokens_used: 0,
            cost: 0.0,
        })
    }

    fn select_fastest_profile(&self) -> Option<LlmProfile> {
        Some(default_llm_profile())
    }

    fn is_health_fresh(&self, _model_id: &str) -> bool {
        true
    }

    fn mark_unhealthy(&self, _model_id: &str, _model_name: &str, _error: Option<String>) {}

    fn record_success(&self, _model_id: &str, _model_name: &str) {}
}

/// In-memory vector store backed by a `HashMap`.
pub struct FakeVectorStore {
    records: Mutex<HashMap<String, VectorRecord>>,
}

impl FakeVectorStore {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(HashMap::new()),
        }
    }

    #[allow(dead_code)]
    pub fn insert(&self, record: VectorRecord) {
        self.records
            .lock()
            .unwrap()
            .insert(record.id.clone(), record);
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.records.lock().unwrap().len()
    }
}

impl Default for FakeVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

fn io_err(msg: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::new(std::io::ErrorKind::Other, msg.into()))
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[async_trait]
impl VectorStore for FakeVectorStore {
    async fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    async fn upsert(
        &self,
        record: VectorRecord,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.records
            .lock()
            .unwrap()
            .insert(record.id.clone(), record);
        Ok(())
    }

    async fn upsert_batch(
        &self,
        records: &[VectorRecord],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut map = self.records.lock().unwrap();
        for record in records {
            map.insert(record.id.clone(), record.clone());
        }
        Ok(())
    }

    async fn search(
        &self,
        story_id: &str,
        query_embedding: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let map = self.records.lock().unwrap();
        let mut scored: Vec<(f32, SearchResult)> = map
            .values()
            .filter(|r| r.story_id == story_id)
            .map(|r| {
                let score = cosine_similarity(&query_embedding, &r.embedding);
                (
                    score,
                    SearchResult {
                        id: r.id.clone(),
                        story_id: r.story_id.clone(),
                        chapter_id: r.chapter_id.clone(),
                        chapter_number: r.chapter_number,
                        text: r.text.clone(),
                        score,
                        metadata: r.metadata.clone(),
                    },
                )
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        Ok(scored.into_iter().map(|(_, r)| r).collect())
    }

    async fn text_search(
        &self,
        story_id: &str,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let query_lower = query.to_lowercase();
        let map = self.records.lock().unwrap();
        let mut results: Vec<SearchResult> = map
            .values()
            .filter(|r| r.story_id == story_id && r.text.to_lowercase().contains(&query_lower))
            .map(|r| SearchResult {
                id: r.id.clone(),
                story_id: r.story_id.clone(),
                chapter_id: r.chapter_id.clone(),
                chapter_number: r.chapter_number,
                text: r.text.clone(),
                score: 1.0,
                metadata: r.metadata.clone(),
            })
            .collect();
        results.truncate(top_k);
        Ok(results)
    }

    async fn hybrid_search(
        &self,
        story_id: &str,
        query: &str,
        query_embedding: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let mut vec_results = self.search(story_id, query_embedding, top_k).await?;
        let text_results = self.text_search(story_id, query, top_k).await?;
        let mut by_id: HashMap<String, SearchResult> = HashMap::new();
        for r in vec_results.drain(..) {
            by_id.insert(r.id.clone(), r);
        }
        for r in text_results {
            by_id.entry(r.id.clone()).or_insert(r);
        }
        let mut merged: Vec<SearchResult> = by_id.into_values().collect();
        merged.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        merged.truncate(top_k);
        Ok(merged)
    }

    async fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.records.lock().unwrap().remove(id);
        Ok(())
    }

    async fn delete_chapter(
        &self,
        chapter_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut map = self.records.lock().unwrap();
        let ids_to_remove: Vec<String> = map
            .values()
            .filter(|r| r.chapter_id == chapter_id)
            .map(|r| r.id.clone())
            .collect();
        for id in ids_to_remove {
            map.remove(&id);
        }
        Ok(())
    }

    async fn count(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.records.lock().unwrap().len())
    }
}

/// Fake runtime-contract provider that returns a configurable contract.
pub struct FakeRuntimeContractProvider {
    contracts: Mutex<HashMap<(String, i32), RuntimeContract>>,
    default: Mutex<Option<RuntimeContract>>,
}

impl FakeRuntimeContractProvider {
    pub fn new() -> Self {
        Self {
            contracts: Mutex::new(HashMap::new()),
            default: Mutex::new(None),
        }
    }

    pub fn with_default(mut self, contract: RuntimeContract) -> Self {
        *self.default.lock().unwrap() = Some(contract);
        self
    }

    pub fn set_contract(
        &self,
        story_id: impl Into<String>,
        chapter_number: i32,
        contract: RuntimeContract,
    ) {
        self.contracts
            .lock()
            .unwrap()
            .insert((story_id.into(), chapter_number), contract);
    }
}

impl Default for FakeRuntimeContractProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeContractProvider for FakeRuntimeContractProvider {
    fn get_runtime_contract(
        &self,
        story_id: &str,
        chapter_number: i32,
    ) -> Result<RuntimeContract, AppError> {
        self.contracts
            .lock()
            .unwrap()
            .get(&(story_id.to_string(), chapter_number))
            .cloned()
            .or_else(|| self.default.lock().unwrap().clone())
            .ok_or_else(|| AppError::Internal {
                message: format!(
                    "No fake runtime contract for {}:{}",
                    story_id, chapter_number
                ),
            })
    }
}

/// Build a minimal runtime contract for tests.
pub fn fake_runtime_contract() -> RuntimeContract {
    RuntimeContract {
        master_setting: MasterSettingContract {
            schema_version: "1".to_string(),
            contract_type: "MASTER_SETTING".to_string(),
            generator_version: "0.22.5".to_string(),
            genre: "测试".to_string(),
            core_tone: "紧张".to_string(),
            pacing_strategy: "正常".to_string(),
            anti_patterns: vec![],
            world_rules: vec!["测试规则".to_string()],
        },
        chapter_contract: None,
    }
}

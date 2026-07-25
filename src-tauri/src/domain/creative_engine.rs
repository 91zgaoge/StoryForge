//! Creative engine aggregate port.
//!
//! A single neutral trait that exposes all creative-engine capabilities used by
//! the `agents` module.  This lets agents depend on `domain` types only and
//! breaks the agents <-> creative_engine cycle.

use async_trait::async_trait;
use tauri::AppHandle;

use crate::{
    domain::{
        adaptive::GenerationStrategy,
        agent_context::NarrativeStructureContext,
        asset_snapshot::AssetSnapshot,
        continuity::ConsistencyCheck,
        contracts::RuntimeContract,
        methodology::MethodologyConfig,
        prompt_synthesis::{AssetManifest, SynthesisResult},
        style::{CraftSliderHint, SanitizeOutcome, StyleBlendConfig, StyleCheckResult, StyleDNA},
        write_time_bundle::WriteTimeBundle,
    },
    error::AppError,
};

/// Port for loading the runtime contract used by the creative engine.
///
/// Implementations live outside of `creative_engine` (e.g. in `story_system`),
/// breaking the `creative_engine -> story_system` dependency cycle.
pub trait RuntimeContractProvider: Send + Sync {
    fn get_runtime_contract(
        &self,
        story_id: &str,
        chapter_number: i32,
    ) -> Result<RuntimeContract, AppError>;
}

/// Port for reading unresolved foreshadowing hints without depending on the
/// concrete `creative_engine` implementation.
pub trait ForeshadowingPort: Send + Sync {
    fn get_writing_hints(&self, story_id: &str, limit: usize) -> Result<Vec<String>, AppError>;
}

/// Port for detecting overdue payoffs without depending on the concrete
/// `creative_engine` implementation.
pub trait PayoffLedgerPort: Send + Sync {
    fn detect_overdue_payoffs(&self, story_id: &str) -> Result<Vec<String>, AppError>;
}

#[async_trait]
pub trait CreativeEnginePort: Send + Sync {
    // ---- WriteTimeBundle ----
    fn load_write_time_bundle(
        &self,
        story_id: &str,
        chapter_number: i32,
        style_slice_override: Option<String>,
        secondary_genre_profile_ids: Option<Vec<String>>,
    ) -> Result<WriteTimeBundle, AppError>;

    fn render_bundle_prompt(&self, bundle: &WriteTimeBundle) -> String;

    // ---- Adaptive generator ----
    async fn build_adaptive_strategy(
        &self,
        story_id: &str,
        base_temperature: Option<f32>,
        story_progress: Option<&str>,
        scene_stage: Option<&str>,
    ) -> Result<GenerationStrategy, AppError>;

    // ---- Continuity engine ----
    fn check_scene_continuity(
        &self,
        story_id: &str,
        scene_id: &str,
        content: &str,
    ) -> Result<ConsistencyCheck, AppError>;

    // ---- Methodology engine ----
    fn build_methodology_prompt_extension(&self, config: &MethodologyConfig) -> String;

    // ---- Prompt personalizer ----
    async fn build_prompt_personalizer_extension(&self, story_id: &str)
        -> Result<String, AppError>;

    // ---- Style guard ----
    fn sanitize_style_brief(&self, text: &str) -> SanitizeOutcome;
    fn default_craft_sliders(&self) -> Vec<CraftSliderHint>;
    fn render_craft_sliders(&self, sliders: &[CraftSliderHint]) -> String;

    // ---- Asset snapshot ----
    fn load_asset_snapshot(&self, story_id: &str, style_dna_id: Option<&str>) -> AssetSnapshot;

    // ---- Style checker ----
    fn check_style(&self, text: &str, target: &StyleDNA) -> StyleCheckResult;
    fn check_style_blend(
        &self,
        text: &str,
        blend: &StyleBlendConfig,
        dnas: &[StyleDNA],
    ) -> StyleCheckResult;

    // ---- Prompt synthesis (TriShot) ----
    fn build_asset_manifest(&self, bundle: &WriteTimeBundle) -> AssetManifest;

    async fn synthesize_prompt(
        &self,
        app_handle: AppHandle,
        instruction: &str,
        current_content_preview: Option<&str>,
        manifest: &AssetManifest,
        bundle_prompt: &str,
        asset_capability_summary: Option<&str>,
    ) -> SynthesisResult;

    async fn refine_prompt(
        &self,
        app_handle: AppHandle,
        synthesized_prompt: &str,
        refinement_focus: Option<&str>,
        story_title: &str,
        story_genre: Option<&str>,
        story_tone: Option<&str>,
    ) -> String;

    // ---- Context builder ----
    async fn build_narrative_structure_context(
        &self,
        story_id: &str,
        chapter_number: Option<i32>,
    ) -> Result<NarrativeStructureContext, AppError>;

    async fn fetch_active_threads(&self, story_id: &str) -> Result<Vec<String>, AppError>;

    async fn build_narrative_event_history(
        &self,
        story_id: &str,
    ) -> Result<Option<String>, AppError>;

    // ---- Foreshadowing ----
    fn get_foreshadowing_hints(
        &self,
        story_id: &str,
        limit: usize,
    ) -> Result<Vec<String>, AppError>;
}

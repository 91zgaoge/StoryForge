use crate::state::StateManager;
use crate::skills::SkillLoader;
use super::analyzer::EvolutionReport;
use crate::error::Result;

pub struct SkillUpdater {
    state_manager: StateManager,
    skill_loader: SkillLoader,
}

impl SkillUpdater {
    pub fn new(
        state_manager: StateManager,
        skill_loader: SkillLoader,
    ) -> Self {
        Self {
            state_manager,
            skill_loader,
        }
    }
    
    pub async fn apply_evolution(
        &self,
        report: &EvolutionReport,
    ) -> Result<()> {
        // Apply new traits
        for new_trait in &report.new_traits_discovered {
            self.state_manager.add_character_trait(
                &new_trait.character_id,
                &new_trait.trait_desc,
                report.chapter_analyzed,
                new_trait.confidence,
                &new_trait.evidence,
            )?;
        }
        
        // Apply style adjustments
        for adjustment in &report.style_adjustments {
            self.state_manager.adjust_style(
                &adjustment.parameter,
                &adjustment.new_value,
                &adjustment.reason,
            )?;
        }
        
        Ok(())
    }
}

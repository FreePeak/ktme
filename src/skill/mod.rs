pub mod action;
pub mod config;
pub mod trigger;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub use action::{Action, ActionType, SkillExecutor};
pub use config::{SkillExecutor as Executor, Trigger, TriggerContext, TriggerType};
pub use trigger::{SkillMatcher, Trigger as SkillTrigger};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub triggers: Vec<Trigger>,
    pub actions: Vec<Action>,
}

impl Skill {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            enabled: true,
            triggers: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn matches(&self, context: &TriggerContext) -> bool {
        let matcher = SkillMatcher::new(self.triggers.clone());
        matcher.matches(context)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillConfig {
    pub skills: Vec<Skill>,
}

impl SkillConfig {
    pub fn load(path: &PathBuf) -> Result<Self, crate::error::KtmeError> {
        let content = fs::read_to_string(path)?;
        let config: SkillConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), crate::error::KtmeError> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add_skill(&mut self, skill: Skill) {
        self.skills.push(skill);
    }

    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.name == name)
    }

    pub fn get_skill_mut(&mut self, name: &str) -> Option<&mut Skill> {
        self.skills.iter_mut().find(|s| s.name == name)
    }

    pub fn find_matching_skills(&self, context: &TriggerContext) -> Vec<&Skill> {
        self.skills
            .iter()
            .filter(|s| s.enabled && s.matches(context))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_config_default() {
        let config = SkillConfig::default();
        assert!(config.skills.is_empty());
    }

    #[test]
    fn test_skill_creation() {
        let skill = Skill::new("test-skill".to_string());
        assert_eq!(skill.name, "test-skill");
        assert!(skill.enabled);
        assert!(skill.triggers.is_empty());
        assert!(skill.actions.is_empty());
    }

    #[test]
    fn test_skill_toml_serialization() {
        let skill = Skill {
            name: "test".to_string(),
            description: Some("Test skill".to_string()),
            enabled: true,
            triggers: vec![Trigger {
                trigger_type: TriggerType::CommitMessage,
                pattern: Some("feat:".to_string()),
                patterns: None,
                regex: Some(false),
            }],
            actions: vec![Action {
                action_type: ActionType::AiGenerate,
                template: Some("Generate docs".to_string()),
                update_tree: None,
                service: None,
                command: None,
            }],
        };

        let toml = toml::to_string(&skill).unwrap();
        assert!(toml.contains("test"));
        assert!(toml.contains("ai_generate"));
    }

    #[test]
    fn test_skill_toml_deserialization() {
        let toml_str = r#"
name = "test-skill"
description = "A test skill"
enabled = true

[[triggers]]
type = "commit_message"
pattern = "feat:"

[[actions]]
type = "ai_generate"
template = "Generate docs"
"#;

        let skill: Skill = toml::from_str(toml_str).unwrap();
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description, Some("A test skill".to_string()));
        assert!(skill.enabled);
        assert_eq!(skill.triggers.len(), 1);
        assert_eq!(skill.actions.len(), 1);
    }

    #[test]
    fn test_skill_config_load_save() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("test_skills.toml");

        let config = SkillConfig {
            skills: vec![Skill::new("test".to_string())],
        };

        config.save(&config_path).unwrap();

        let loaded = SkillConfig::load(&config_path).unwrap();
        assert_eq!(loaded.skills.len(), 1);
        assert_eq!(loaded.skills[0].name, "test");

        std::fs::remove_file(&config_path).ok();
    }
}

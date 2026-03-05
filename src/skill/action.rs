use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    AiGenerate,
    UpdateTree,
    Sync,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: ActionType,
    pub template: Option<String>,
    pub update_tree: Option<UpdateTreeConfig>,
    pub service: Option<String>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTreeConfig {
    pub add_features: Option<Vec<String>>,
    pub remove_features: Option<Vec<String>>,
    pub add_relations: Option<Vec<FeatureRelation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureRelation {
    pub from: String,
    pub to: String,
    pub relation_type: String,
}

pub struct SkillExecutor;

impl SkillExecutor {
    pub async fn execute(
        action: &Action,
        context: &super::TriggerContext,
    ) -> Result<String, String> {
        match action.action_type {
            ActionType::AiGenerate => Self::execute_ai_generate(action, context).await,
            ActionType::UpdateTree => Self::execute_update_tree(action, context).await,
            ActionType::Sync => Self::execute_sync(action, context).await,
            ActionType::Custom => Self::execute_custom(action, context).await,
        }
    }

    async fn execute_ai_generate(
        action: &Action,
        _context: &super::TriggerContext,
    ) -> Result<String, String> {
        let template = action
            .template
            .as_deref()
            .unwrap_or("Generate documentation for the changes");
        tracing::info!("AI Generate action: template={}", template);
        Ok(format!(
            "AI generation triggered with template: {}",
            template
        ))
    }

    async fn execute_update_tree(
        action: &Action,
        _context: &super::TriggerContext,
    ) -> Result<String, String> {
        let config = action
            .update_tree
            .as_ref()
            .ok_or("Missing update_tree config")?;

        if let Some(features) = &config.add_features {
            tracing::info!("Adding features: {:?}", features);
        }
        if let Some(features) = &config.remove_features {
            tracing::info!("Removing features: {:?}", features);
        }
        if let Some(relations) = &config.add_relations {
            tracing::info!("Adding relations: {:?}", relations);
        }

        Ok("Tree update completed".to_string())
    }

    async fn execute_sync(
        action: &Action,
        _context: &super::TriggerContext,
    ) -> Result<String, String> {
        let service = action.service.as_deref().unwrap_or("default");
        tracing::info!("Syncing service: {}", service);
        Ok(format!("Sync completed for service: {}", service))
    }

    async fn execute_custom(
        action: &Action,
        _context: &super::TriggerContext,
    ) -> Result<String, String> {
        let command = action.command.as_ref().ok_or("Missing command")?;
        tracing::info!("Executing custom command: {}", command);
        Ok(format!("Custom command executed: {}", command))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_type_serialization() {
        let action = Action {
            action_type: ActionType::AiGenerate,
            template: Some("Generate docs".to_string()),
            update_tree: None,
            service: None,
            command: None,
        };

        let toml = toml::to_string(&action).unwrap();
        assert!(toml.contains("ai_generate"));
    }

    #[test]
    fn test_update_tree_config() {
        let config = UpdateTreeConfig {
            add_features: Some(vec!["feature1".to_string(), "feature2".to_string()]),
            remove_features: None,
            add_relations: Some(vec![FeatureRelation {
                from: "a".to_string(),
                to: "b".to_string(),
                relation_type: "depends_on".to_string(),
            }]),
        };

        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("feature1"));
    }

    #[tokio::test]
    async fn test_execute_ai_generate() {
        let action = Action {
            action_type: ActionType::AiGenerate,
            template: Some("Generate documentation".to_string()),
            update_tree: None,
            service: None,
            command: None,
        };

        let context = super::super::TriggerContext::new(
            vec!["src/lib.rs".to_string()],
            "feat: add feature".to_string(),
        );

        let result = SkillExecutor::execute(&action, &context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_custom() {
        let action = Action {
            action_type: ActionType::Custom,
            template: None,
            update_tree: None,
            service: None,
            command: Some("echo hello".to_string()),
        };

        let context =
            super::super::TriggerContext::new(vec!["src/lib.rs".to_string()], "test".to_string());

        let result = SkillExecutor::execute(&action, &context).await;
        assert!(result.is_ok());
    }
}

use glob::glob;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    FilePattern,
    CommitMessage,
    FileContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    #[serde(rename = "type")]
    pub trigger_type: TriggerType,
    pub pattern: Option<String>,
    pub patterns: Option<Vec<String>>,
    pub regex: Option<bool>,
}

impl Trigger {
    pub fn matches(&self, context: &TriggerContext) -> bool {
        match self.trigger_type {
            TriggerType::FilePattern => self.matches_file_pattern(context),
            TriggerType::CommitMessage => self.matches_commit_message(context),
            TriggerType::FileContent => self.matches_file_content(context),
        }
    }

    fn matches_file_pattern(&self, context: &TriggerContext) -> bool {
        let patterns = match &self.pattern {
            Some(p) => vec![p.clone()],
            None => self.patterns.clone().unwrap_or_default(),
        };

        for pattern in patterns {
            for file in &context.changed_files {
                if glob_match(&pattern, file) {
                    return true;
                }
            }
        }
        false
    }

    fn matches_commit_message(&self, context: &TriggerContext) -> bool {
        let message = &context.commit_message;
        let is_regex = self.regex.unwrap_or(false);

        if is_regex {
            if let Some(pattern) = &self.pattern {
                if let Ok(re) = Regex::new(pattern) {
                    return re.is_match(message);
                }
            }
            if let Some(patterns) = &self.patterns {
                for pattern in patterns {
                    if let Ok(re) = Regex::new(pattern) {
                        if re.is_match(message) {
                            return true;
                        }
                    }
                }
            }
        } else {
            let search_patterns: Vec<String> = match &self.pattern {
                Some(p) => vec![p.clone()],
                None => self.patterns.clone().unwrap_or_default(),
            };

            for pattern in search_patterns {
                if message.to_lowercase().contains(&pattern.to_lowercase()) {
                    return true;
                }
            }
        }
        false
    }

    fn matches_file_content(&self, context: &TriggerContext) -> bool {
        let is_regex = self.regex.unwrap_or(false);
        let patterns: Vec<String> = match &self.pattern {
            Some(p) => vec![p.clone()],
            None => self.patterns.clone().unwrap_or_default(),
        };

        for pattern in &patterns {
            if is_regex {
                if let Ok(re) = Regex::new(pattern) {
                    for file in &context.changed_files {
                        if let Ok(content) = std::fs::read_to_string(file) {
                            if re.is_match(&content) {
                                return true;
                            }
                        }
                    }
                }
            } else {
                for file in &context.changed_files {
                    if let Ok(content) = std::fs::read_to_string(file) {
                        if content.contains(pattern) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

fn glob_match(pattern: &str, path: &str) -> bool {
    if let Ok(globber) = glob::Pattern::new(pattern) {
        return globber.matches(path);
    }

    let pattern_simple = pattern.replace("**/", "*");
    let parts: Vec<&str> = pattern_simple.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();

    let mut pattern_idx = 0;
    let mut path_idx = 0;

    while pattern_idx < parts.len() || path_idx < path_parts.len() {
        let p = parts.get(pattern_idx);
        let f = path_parts.get(path_idx);

        match (p, f) {
            (Some(p), Some(_f)) if *p == "*" => {
                path_idx += 1;
                pattern_idx += 1;
            }
            (Some(p), Some(_f)) if *p == "**" => {
                if pattern_idx + 1 >= parts.len() {
                    return true;
                }
                pattern_idx += 1;
            }
            (Some(p), Some(f)) if p == f => {
                pattern_idx += 1;
                path_idx += 1;
            }
            (None, Some(_)) => return true,
            _ => return false,
        }
    }
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TriggerContext {
    pub changed_files: Vec<String>,
    pub commit_message: String,
    pub service: Option<String>,
}

impl TriggerContext {
    pub fn new(changed_files: Vec<String>, commit_message: String) -> Self {
        Self {
            changed_files,
            commit_message,
            service: None,
        }
    }

    pub fn with_service(mut self, service: String) -> Self {
        self.service = Some(service);
        self
    }
}

pub struct SkillMatcher {
    triggers: Vec<Trigger>,
}

impl SkillMatcher {
    pub fn new(triggers: Vec<Trigger>) -> Self {
        Self { triggers }
    }

    pub fn matches(&self, context: &TriggerContext) -> bool {
        self.triggers.iter().any(|t| t.matches(context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_file_pattern_glob() {
        let trigger = Trigger {
            trigger_type: TriggerType::FilePattern,
            pattern: Some("src/**/*.rs".to_string()),
            patterns: None,
            regex: Some(false),
        };

        let context = TriggerContext::new(
            vec!["src/lib.rs".to_string(), "src/main.rs".to_string()],
            "test commit".to_string(),
        );

        assert!(trigger.matches(&context));
    }

    #[test]
    fn test_trigger_commit_message_exact() {
        let trigger = Trigger {
            trigger_type: TriggerType::CommitMessage,
            pattern: Some("feat:".to_string()),
            patterns: None,
            regex: Some(false),
        };

        let context = TriggerContext::new(
            vec!["src/lib.rs".to_string()],
            "feat: add new feature".to_string(),
        );

        assert!(trigger.matches(&context));
    }

    #[test]
    fn test_trigger_commit_message_regex() {
        let trigger = Trigger {
            trigger_type: TriggerType::CommitMessage,
            pattern: Some(r"^feat(\(.+\))?:".to_string()),
            patterns: None,
            regex: Some(true),
        };

        let context = TriggerContext::new(
            vec!["src/lib.rs".to_string()],
            "feat(auth): add login".to_string(),
        );

        assert!(trigger.matches(&context));
    }

    #[test]
    fn test_skill_matcher_or_logic() {
        let triggers = vec![
            Trigger {
                trigger_type: TriggerType::CommitMessage,
                pattern: Some("feat:".to_string()),
                patterns: None,
                regex: Some(false),
            },
            Trigger {
                trigger_type: TriggerType::FilePattern,
                pattern: Some("docs/**".to_string()),
                patterns: None,
                regex: Some(false),
            },
        ];

        let matcher = SkillMatcher::new(triggers);

        let context1 = TriggerContext::new(
            vec!["src/lib.rs".to_string()],
            "feat: add feature".to_string(),
        );
        assert!(matcher.matches(&context1));

        let context2 = TriggerContext::new(
            vec!["docs/README.md".to_string()],
            "chore: update docs".to_string(),
        );
        assert!(matcher.matches(&context2));
    }

    #[test]
    fn test_trigger_context_with_service() {
        let context = TriggerContext::new(vec!["src/lib.rs".to_string()], "test".to_string())
            .with_service("my-service".to_string());

        assert_eq!(context.service, Some("my-service".to_string()));
    }
}

use std::collections::HashMap;
use std::sync::Arc;

use crate::config::{Config, LlmConfig, ResolvedModelProfile};
use crate::error::{Error, Result};
use crate::llm::{LlmProvider, create_provider};

/// LLM 提供者注册表
///
/// 管理多个模型及其对应的 `LlmProvider` 实例。
/// 它是 Phase 4 多模型编排系统的核心组件。
pub struct LlmProviderRegistry {
    /// 模型名称 -> 提供者实例
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    /// 模型名称 -> 已解析的模型元数据（包含价格、能力等级等）
    profiles: HashMap<String, ResolvedModelProfile>,
}

impl std::fmt::Debug for LlmProviderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmProviderRegistry")
            .field("providers", &self.providers.keys().collect::<Vec<_>>())
            .field("profiles", &self.profiles)
            .finish()
    }
}

impl LlmProviderRegistry {
    /// 从配置创建注册表
    ///
    /// 此方法会预先解析所有模型档案并创建对应的提供者实例。
    /// 如果任何一个档案解析失败或提供者创建失败，将返回错误。
    pub fn from_config(config: &Config) -> Result<Self> {
        let mut providers = HashMap::new();
        let mut profiles = HashMap::new();

        // 1. 始终解析并添加 "default" 提供者（基于 config.llm）
        let default_profile = config.resolve_model_profile("default")?;
        let default_provider = create_provider(Self::profile_to_llm_config(&default_profile));

        providers.insert("default".to_string(), default_provider.clone());
        profiles.insert("default".to_string(), default_profile.clone());

        // 如果用户定义的 default_model 不是 "default"，也映射过去
        if config.default_model != "default" {
            providers.insert(config.default_model.clone(), default_provider);
            profiles.insert(config.default_model.clone(), default_profile);
        }

        // 2. 遍历并创建所有命名的模型档案
        for profile_name in config.models.keys() {
            // "default" 已经处理过了，如果是保留名称则跳过
            if profile_name == "default" {
                continue;
            }

            let resolved = config.resolve_model_profile(profile_name)?;
            let provider = create_provider(Self::profile_to_llm_config(&resolved));

            providers.insert(profile_name.clone(), provider);
            profiles.insert(profile_name.clone(), resolved);
        }

        Ok(Self {
            providers,
            profiles,
        })
    }

    /// 获取指定的模型提供者
    pub fn get_provider(&self, model_name: &str) -> Result<Arc<dyn LlmProvider>> {
        self.providers
            .get(model_name)
            .cloned()
            .ok_or_else(|| Error::ModelProfileNotFound(model_name.to_string()))
    }

    /// 获取默认的模型提供者
    pub fn default_provider(&self) -> Arc<dyn LlmProvider> {
        self.providers
            .get("default")
            .cloned()
            .expect("Default provider must exist")
    }

    /// 获取模型元数据
    pub fn get_model_profile(&self, model_name: &str) -> Option<&ResolvedModelProfile> {
        self.profiles.get(model_name)
    }

    /// 列出所有可用的模型名称
    pub fn list_available_models(&self) -> Vec<String> {
        let mut models: Vec<String> = self.profiles.keys().cloned().collect();
        models.sort();
        models
    }

    /// 辅助方法：将已解析的模型档案转换为 LlmConfig，以便调用工厂函数
    fn profile_to_llm_config(profile: &ResolvedModelProfile) -> LlmConfig {
        LlmConfig {
            provider: profile.provider.clone(),
            api_key: profile.api_key.clone(),
            base_url: profile.base_url.clone(),
            model: profile.model.clone(),
            max_tokens: profile.max_tokens,
            temperature: profile.temperature,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ModelProfile;
    use std::collections::HashMap;

    fn create_mock_config() -> Config {
        let mut config = Config::default();
        config.llm.model = "gpt-4o-default".to_string();
        config.llm.api_key = "default-key".to_string();
        config
    }

    #[test]
    fn test_registry_with_default_only() -> Result<()> {
        let config = create_mock_config();
        let registry = LlmProviderRegistry::from_config(&config)?;

        assert!(registry.providers.contains_key("default"));
        assert_eq!(registry.list_available_models(), vec!["default"]);

        let profile = registry.get_model_profile("default").unwrap();
        assert_eq!(profile.model, "gpt-4o-default");

        Ok(())
    }

    #[test]
    fn test_registry_with_named_profiles() -> Result<()> {
        let mut config = create_mock_config();

        let mut models = HashMap::new();
        models.insert(
            "gpt-4o-mini".to_string(),
            ModelProfile {
                provider: "openai".to_string(),
                model: "gpt-4o-mini".to_string(),
                api_key: None,  // inherit
                base_url: None, // inherit
                context_window: Some(128000),
                cost_per_1k_input: Some(0.00015),
                cost_per_1k_output: Some(0.0006),
                capability_tier: Some("low".to_string()),
                max_tokens: Some(4096),
                temperature: Some(0.7),
            },
        );
        models.insert(
            "o1-preview".to_string(),
            ModelProfile {
                provider: "openai".to_string(),
                model: "o1-preview".to_string(),
                api_key: Some("secret-key".to_string()), // override
                base_url: None,
                context_window: Some(128000),
                cost_per_1k_input: Some(0.015),
                cost_per_1k_output: Some(0.06),
                capability_tier: Some("high".to_string()),
                max_tokens: Some(32768),
                temperature: Some(1.0),
            },
        );
        config.models = models;

        let registry = LlmProviderRegistry::from_config(&config)?;

        let mut available = registry.list_available_models();
        available.sort();
        assert_eq!(available, vec!["default", "gpt-4o-mini", "o1-preview"]);

        // 验证 gpt-4o-mini 模型（继承了 default 的 api_key）
        let cheap = registry.get_model_profile("gpt-4o-mini").unwrap();
        assert_eq!(cheap.model, "gpt-4o-mini");
        assert_eq!(cheap.api_key, "default-key");
        assert_eq!(cheap.capability_tier, Some("low".to_string()));

        // 验证 o1-preview 模型（覆写了 api_key）
        let strong = registry.get_model_profile("o1-preview").unwrap();
        assert_eq!(strong.model, "o1-preview");
        assert_eq!(strong.api_key, "secret-key");
        assert_eq!(strong.capability_tier, Some("high".to_string()));

        // 验证 get_provider
        assert!(registry.get_provider("gpt-4o-mini").is_ok());
        assert!(registry.get_provider("o1-preview").is_ok());
        assert!(registry.get_provider("non-existent").is_err());

        Ok(())
    }

    #[test]
    fn test_registry_custom_default_model_name() -> Result<()> {
        let mut config = create_mock_config();
        config.default_model = "my-fav".to_string();

        let registry = LlmProviderRegistry::from_config(&config)?;

        // 应该既可以用 "default" 也可以用 "my-fav" 获取
        assert!(registry.get_provider("default").is_ok());
        assert!(registry.get_provider("my-fav").is_ok());

        let mut available = registry.list_available_models();
        available.sort();
        assert_eq!(available, vec!["default", "my-fav"]);

        Ok(())
    }

    #[test]
    fn test_registry_get_model_profile() -> Result<()> {
        let mut config = create_mock_config();
        let mut models = HashMap::new();
        models.insert(
            "test-model".to_string(),
            ModelProfile {
                provider: "openai".to_string(),
                model: "test-model".to_string(),
                api_key: None,
                base_url: None,
                context_window: Some(100),
                cost_per_1k_input: Some(1.0),
                cost_per_1k_output: Some(2.0),
                capability_tier: Some("test".to_string()),
                max_tokens: None,
                temperature: None,
            },
        );
        config.models = models;

        let registry = LlmProviderRegistry::from_config(&config)?;
        let profile = registry.get_model_profile("test-model").unwrap();

        assert_eq!(profile.context_window, Some(100));
        assert_eq!(profile.cost_per_1k_input, Some(1.0));
        assert_eq!(profile.cost_per_1k_output, Some(2.0));
        assert_eq!(profile.capability_tier, Some("test".to_string()));

        assert!(registry.get_model_profile("non-existent").is_none());

        Ok(())
    }
}

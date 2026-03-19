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

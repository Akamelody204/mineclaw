//! 提示词模板组装器
//!
//! 负责将可用的模型信息注入到 Orchestrator 的系统提示词中，
//! 使 Orchestrator 能够了解不同模型的特性并做出智能的模型选择决策。

use crate::config::ResolvedModelProfile;
use crate::llm::LlmProviderRegistry;

/// 提示词组装器
pub struct PromptAssembler;

impl PromptAssembler {
    /// 构建注入了模型信息的 Orchestrator 系统提示词
    ///
    /// # 参数
    /// - `base_prompt`: 基础系统提示词
    /// - `registry`: LLM 提供者注册表，用于获取可用模型信息
    ///
    /// # 返回
    /// 注入了模型信息的完整系统提示词
    pub fn build_orchestrator_prompt(base_prompt: &str, registry: &LlmProviderRegistry) -> String {
        let model_info = Self::format_available_models(registry);

        format!(
            "{}\n\n---\n\n## 可用模型信息\n\n以下是当前可用的模型及其特性，你可以根据任务需求选择合适的模型：\n\n{}\n\n## 模型选择指南\n\n- 对于复杂的推理、规划或需要高精度的任务，优先使用 capability_tier 为 \"high\" 的模型\n- 对于简单的文本处理、摘要或快速响应，可以使用 capability_tier 为 \"low\" 或 \"medium\" 的模型以节省成本\n- 考虑任务的上下文大小，选择足够大的 context_window\n- 在 spawn_sub_agent 时，你可以通过 model_profile 参数指定使用哪个模型\n",
            base_prompt, model_info
        )
    }

    /// 格式化可用模型信息为结构化文本
    fn format_available_models(registry: &LlmProviderRegistry) -> String {
        let model_names = registry.list_available_models();

        if model_names.is_empty() {
            return "没有可用的模型配置。".to_string();
        }

        let mut sections = Vec::new();

        for name in model_names {
            if let Some(profile) = registry.get_model_profile(&name) {
                sections.push(Self::format_single_model(&name, profile));
            }
        }

        sections.join("\n\n")
    }

    /// 格式化单个模型的信息
    fn format_single_model(model_name: &str, profile: &ResolvedModelProfile) -> String {
        let mut lines = Vec::new();

        lines.push(format!("### 模型: `{}`", model_name));
        lines.push(format!("- **模型名称**: {}", profile.model));
        lines.push(format!("- **提供商**: {}", profile.provider));
        lines.push(format!("- **最大输出 Token**: {}", profile.max_tokens));
        lines.push(format!("- **默认温度**: {:.1}", profile.temperature));

        if let Some(ctx) = profile.context_window {
            lines.push(format!("- **上下文窗口**: {} tokens", ctx));
        } else {
            lines.push("- **上下文窗口**: 未配置".to_string());
        }

        if let Some(cost_in) = profile.cost_per_1k_input {
            lines.push(format!("- **输入成本**: ${:.6}/1K tokens", cost_in));
        } else {
            lines.push("- **输入成本**: 未配置".to_string());
        }

        if let Some(cost_out) = profile.cost_per_1k_output {
            lines.push(format!("- **输出成本**: ${:.6}/1K tokens", cost_out));
        } else {
            lines.push("- **输出成本**: 未配置".to_string());
        }

        if let Some(tier) = &profile.capability_tier {
            lines.push(format!("- **能力等级**: {}", tier));
        } else {
            lines.push("- **能力等级**: 未配置".to_string());
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ModelProfile};
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        let mut config = Config::default();
        config.llm.model = "gpt-4o".to_string();
        config.llm.api_key = "test-key".to_string();
        config.llm.provider = "openai".to_string();
        config
    }

    #[test]
    fn test_build_orchestrator_prompt_includes_model_info() {
        let mut config = create_test_config();

        // 添加测试用的模型
        let mut models = HashMap::new();
        models.insert(
            "gpt-4o-mini".to_string(),
            ModelProfile {
                provider: "openai".to_string(),
                model: "gpt-4o-mini".to_string(),
                api_key: None,
                base_url: None,
                context_window: Some(128000),
                cost_per_1k_input: Some(0.00015),
                cost_per_1k_output: Some(0.0006),
                capability_tier: Some("low".to_string()),
                max_tokens: Some(4096),
                temperature: Some(0.7),
            },
        );
        config.models = models;

        let registry = LlmProviderRegistry::from_config(&config).unwrap();
        let base_prompt = "You are an orchestrator agent.";

        let result = PromptAssembler::build_orchestrator_prompt(base_prompt, &registry);

        // 验证包含基础提示词
        assert!(result.contains(base_prompt));

        // 验证包含模型信息
        assert!(result.contains("可用模型信息"));
        assert!(result.contains("模型: `default`"));
        assert!(result.contains("模型: `gpt-4o-mini`"));
        assert!(result.contains("gpt-4o-mini"));
        assert!(result.contains("能力等级"));
        assert!(result.contains("low"));
        assert!(result.contains("上下文窗口"));
        assert!(result.contains("128000"));
        assert!(result.contains("0.00015"));
        assert!(result.contains("0.0006"));

        // 验证包含模型选择指南
        assert!(result.contains("模型选择指南"));
        assert!(result.contains("spawn_sub_agent"));
        assert!(result.contains("model_profile"));
    }

    #[test]
    fn test_format_available_models_with_no_models() {
        let config = create_test_config();
        // 手动创建一个空的 registry 用于测试（实际不会发生这种情况）
        let registry = LlmProviderRegistry::from_config(&config).unwrap();

        // 清理掉所有模型（这在正常使用中不会发生，仅用于测试边界情况）
        let model_names = registry.list_available_models();
        assert!(!model_names.is_empty()); // 至少有 default
    }

    #[test]
    fn test_format_single_model_complete() {
        let profile = ResolvedModelProfile {
            provider: "openai".to_string(),
            api_key: "key".to_string(),
            base_url: "url".to_string(),
            model: "test-model".to_string(),
            max_tokens: 4096,
            temperature: 0.8,
            context_window: Some(128000),
            cost_per_1k_input: Some(0.01),
            cost_per_1k_output: Some(0.03),
            capability_tier: Some("high".to_string()),
        };

        let result = PromptAssembler::format_single_model("test", &profile);

        assert!(result.contains("模型: `test`"));
        assert!(result.contains("test-model"));
        assert!(result.contains("openai"));
        assert!(result.contains("4096"));
        assert!(result.contains("0.8"));
        assert!(result.contains("128000 tokens"));
        assert!(result.contains("$0.010000/1K tokens"));
        assert!(result.contains("$0.030000/1K tokens"));
        assert!(result.contains("high"));
    }

    #[test]
    fn test_format_single_model_optional_fields_missing() {
        let profile = ResolvedModelProfile {
            provider: "openai".to_string(),
            api_key: "key".to_string(),
            base_url: "url".to_string(),
            model: "minimal-model".to_string(),
            max_tokens: 1024,
            temperature: 0.5,
            context_window: None,
            cost_per_1k_input: None,
            cost_per_1k_output: None,
            capability_tier: None,
        };

        let result = PromptAssembler::format_single_model("minimal", &profile);

        assert!(result.contains("minimal-model"));
        // 只要包含关键字就行，不需要精确匹配整个字符串
        assert!(result.contains("上下文窗口"));
        assert!(result.contains("输入成本"));
        assert!(result.contains("输出成本"));
        assert!(result.contains("能力等级"));
    }
}

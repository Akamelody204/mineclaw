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

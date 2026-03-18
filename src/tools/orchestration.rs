//! 协作与总控工具模块
//!
//! 提供 Agent 之间的协作、任务指派和工单提交等功能。

use super::{LocalTool, ToolContext};
use crate::error::Result;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::fmt::Debug;

// ==================== OrchestrationInterface ====================

/// 总控与协作接口 trait
///
/// 通过此接口，本地工具可以调用总控的能力（如创建子 Agent、派发任务、提交工单等），
/// 从而实现模块间的解耦。
#[async_trait]
pub trait OrchestrationInterface: Send + Sync + Debug {
    /// 提交汇报工单（向上一级汇报或接力转交）
    ///
    /// 遵循 PLAN.md 规范：包含已完成详情、相关文件列表和下一步计划。
    async fn submit_report_work_order(
        &self,
        completed_details: &str,
        related_files: &[String],
        next_stage_plan: &str,
    ) -> Result<Value>;

    /// 提交求助工单（触发 CMA 干预）
    async fn submit_help_work_order(
        &self,
        problem_description: &str,
        current_status: &str,
    ) -> Result<Value>;

    /// 创建子 Agent
    ///
    /// 返回新创建的 agent_id
    async fn spawn_sub_agent(&self, name: &str, role: &str, capability: &str) -> Result<String>;

    /// 指派任务给指定 Agent
    ///
    /// # 参数
    /// * `target_agent_id` - 目标 Agent 的唯一标识
    /// * `instruction` - 具体的任务要求或工单指令内容
    /// * `is_parallel` - 是否采用并行执行模式（非阻塞）
    async fn assign_task(
        &self,
        target_agent_id: &str,
        instruction: &str,
        is_parallel: bool,
    ) -> Result<Value>;
}

// ==================== Tools Implementation ====================

/// 提交工单工具
pub struct SubmitWorkOrderTool;

#[async_trait]
impl LocalTool for SubmitWorkOrderTool {
    fn name(&self) -> &str {
        "submit_work_order"
    }

    fn description(&self) -> &str {
        "提交工单向上一级 Agent 反馈任务完成情况和后续计划。当你完成当前阶段任务或需要接力转交时使用。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "completed_details": {
                    "type": "string",
                    "description": "已完成部分的详细标注"
                },
                "related_files": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "相关文件的相对路径列表"
                },
                "next_stage_plan": {
                    "type": "string",
                    "description": "下一阶段的详细计划"
                }
            },
            "required": ["completed_details", "related_files", "next_stage_plan"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let orchestrator = context.orchestrator.ok_or_else(|| {
            crate::error::Error::WorkOrder(
                "当前工具上下文中不存在总控接口，无法提交工单".to_string(),
            )
        })?;

        let completed = arguments["completed_details"].as_str().unwrap_or_default();
        let files: Vec<String> = arguments["related_files"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        let next_plan = arguments["next_stage_plan"].as_str().unwrap_or_default();

        orchestrator
            .submit_report_work_order(completed, &files, next_plan)
            .await
    }
}

/// 主动求助工具
pub struct RequestHelpTool;

#[async_trait]
impl LocalTool for RequestHelpTool {
    fn name(&self) -> &str {
        "request_help"
    }

    fn description(&self) -> &str {
        "遇到无法解决的困难或歧义时，主动请求 CMA（上下文管理 Agent）干预。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "problem_description": {
                    "type": "string",
                    "description": "遇到的具体困难或阻塞原因"
                },
                "current_status": {
                    "type": "string",
                    "description": "当前已尝试的操作和进度状态"
                }
            },
            "required": ["problem_description", "current_status"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let orchestrator = context.orchestrator.ok_or_else(|| {
            crate::error::Error::WorkOrder(
                "当前工具上下文中不存在总控接口，无法发起求助".to_string(),
            )
        })?;

        let problem = arguments["problem_description"]
            .as_str()
            .unwrap_or_default();
        let status = arguments["current_status"].as_str().unwrap_or_default();

        orchestrator.submit_help_work_order(problem, status).await
    }
}

// ==================== Downward Control Tools ====================

/// 创建子 Agent 工具
pub struct SpawnSubAgentTool;

#[async_trait]
impl LocalTool for SpawnSubAgentTool {
    fn name(&self) -> &str {
        "spawn_sub_agent"
    }

    fn description(&self) -> &str {
        "创建一个新的子 Agent 来处理特定的子任务。返回新创建的 agent_id。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Agent 的名称"
                },
                "role": {
                    "type": "string",
                    "description": "Agent 的角色定义（例如：rust_developer, tester）"
                },
                "capability": {
                    "type": "string",
                    "description": "简短的能力描述"
                }
            },
            "required": ["role", "capability"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let orchestrator = context.orchestrator.ok_or_else(|| {
            crate::error::Error::AgentExecution(
                "当前工具上下文中不存在总控接口，无法创建子 Agent".to_string(),
            )
        })?;

        let name = arguments["name"].as_str().unwrap_or("sub_agent");
        let role = arguments["role"].as_str().unwrap_or_default();
        let capability = arguments["capability"].as_str().unwrap_or_default();

        let agent_id = orchestrator.spawn_sub_agent(name, role, capability).await?;
        Ok(json!({ "agent_id": agent_id }))
    }
}

/// 指派任务工具
pub struct AssignTaskTool;

#[async_trait]
impl LocalTool for AssignTaskTool {
    fn name(&self) -> &str {
        "assign_task"
    }

    fn description(&self) -> &str {
        "将任务指派给指定的下级 Agent。你可以选择串行等待结果，或者并行执行。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "target_agent_id": {
                    "type": "string",
                    "description": "目标 Agent 的 ID"
                },
                "instruction": {
                    "type": "string",
                    "description": "具体的任务指令内容"
                },
                "is_parallel": {
                    "type": "boolean",
                    "description": "是否并行执行（非阻塞）",
                    "default": false
                }
            },
            "required": ["target_agent_id", "instruction"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let orchestrator = context.orchestrator.ok_or_else(|| {
            crate::error::Error::AgentExecution(
                "当前工具上下文中不存在总控接口，无法指派任务".to_string(),
            )
        })?;

        let target_id = arguments["target_agent_id"].as_str().unwrap_or_default();
        let instruction = arguments["instruction"].as_str().unwrap_or_default();
        let is_parallel = arguments["is_parallel"].as_bool().unwrap_or(false);

        orchestrator
            .assign_task(target_id, instruction, is_parallel)
            .await
    }
}

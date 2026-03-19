//! 工单系统
//!
//! 提供 Agent 之间的通信机制，包括任务完成汇报、接力转交、求助等。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::agent::types::AgentId;

// ============================================================================
// WorkOrderId - 工单唯一标识
// ============================================================================

/// 工单唯一标识
///
/// 使用 Uuid v4 作为底层实现，提供类型安全的包装。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkOrderId(Uuid);

impl WorkOrderId {
    /// 创建一个新的随机 WorkOrderId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 从 Uuid 创建 WorkOrderId
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// 获取底层的 Uuid
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// 从字符串解析 WorkOrderId
    pub fn parse_str(s: &str) -> Result<Self, String> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|e| format!("Invalid WorkOrderId: {}", e))
    }
}

impl Default for WorkOrderId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for WorkOrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// WorkOrderType - 工单类型
// ============================================================================

/// 工单类型
///
/// 定义工单的用途和类型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkOrderType {
    /// 任务完成汇报
    TaskCompletion,
    /// 接力转交
    Handover,
    /// 求助
    HelpRequest,
    /// 状态更新
    StatusUpdate,
}

impl fmt::Display for WorkOrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaskCompletion => write!(f, "TaskCompletion"),
            Self::Handover => write!(f, "Handover"),
            Self::HelpRequest => write!(f, "HelpRequest"),
            Self::StatusUpdate => write!(f, "StatusUpdate"),
        }
    }
}

// ============================================================================
// WorkOrderRecipient - 工单接收对象
// ============================================================================

/// 工单接收对象
///
/// 定义工单的接收目标。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkOrderRecipient {
    /// 发送给上下文管理 Agent (CMA)
    ContextManager,
    /// 发送给指定总控
    Orchestrator(AgentId),
}

impl fmt::Display for WorkOrderRecipient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContextManager => write!(f, "ContextManager"),
            Self::Orchestrator(id) => write!(f, "Orchestrator({})", id),
        }
    }
}

// ============================================================================
// WorkOrder - 工单
// ============================================================================

/// 工单
///
/// 代表 Agent 之间传递的工作单元。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOrder {
    /// 工单唯一标识
    pub id: WorkOrderId,
    /// 工单类型
    pub work_order_type: WorkOrderType,
    /// 接收对象
    pub recipient: WorkOrderRecipient,
    /// 会话 ID
    pub session_id: Uuid,
    /// 工单标题
    pub title: String,
    /// 工单内容（JSON 或自由文本）
    pub content: String,
    /// 相关文件
    pub related_files: Vec<String>,
    /// 建议回退的 Checkpoint
    pub suggested_checkpoint_id: Option<String>,
    /// 创建者（Agent 或总控）
    pub created_by: Option<AgentId>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl WorkOrder {
    /// 创建新的工单
    ///
    /// # 参数
    /// * `work_order_type` - 工单类型
    /// * `recipient` - 接收对象
    /// * `session_id` - 会话 ID
    /// * `title` - 工单标题
    /// * `content` - 工单内容
    ///
    /// # 返回
    /// 返回新创建的工单
    pub fn new(
        work_order_type: WorkOrderType,
        recipient: WorkOrderRecipient,
        session_id: Uuid,
        title: String,
        content: String,
    ) -> Self {
        Self {
            id: WorkOrderId::new(),
            work_order_type,
            recipient,
            session_id,
            title,
            content,
            related_files: Vec::new(),
            suggested_checkpoint_id: None,
            created_by: None,
            created_at: Utc::now(),
        }
    }

    /// 创建任务完成汇报工单
    pub fn task_completion(
        recipient: WorkOrderRecipient,
        session_id: Uuid,
        title: String,
        content: String,
    ) -> Self {
        Self::new(
            WorkOrderType::TaskCompletion,
            recipient,
            session_id,
            title,
            content,
        )
    }

    /// 创建接力转交工单
    pub fn handover(
        recipient: WorkOrderRecipient,
        session_id: Uuid,
        title: String,
        content: String,
    ) -> Self {
        Self::new(
            WorkOrderType::Handover,
            recipient,
            session_id,
            title,
            content,
        )
    }

    /// 创建求助工单
    pub fn help_request(
        recipient: WorkOrderRecipient,
        session_id: Uuid,
        title: String,
        content: String,
    ) -> Self {
        Self::new(
            WorkOrderType::HelpRequest,
            recipient,
            session_id,
            title,
            content,
        )
    }

    /// 创建状态更新工单
    pub fn status_update(
        recipient: WorkOrderRecipient,
        session_id: Uuid,
        title: String,
        content: String,
    ) -> Self {
        Self::new(
            WorkOrderType::StatusUpdate,
            recipient,
            session_id,
            title,
            content,
        )
    }

    /// 添加相关文件
    pub fn with_related_file(mut self, file: String) -> Self {
        self.related_files.push(file);
        self
    }

    /// 设置多个相关文件
    pub fn with_related_files(mut self, files: Vec<String>) -> Self {
        self.related_files = files;
        self
    }

    /// 设置建议的 Checkpoint ID
    pub fn with_suggested_checkpoint(mut self, checkpoint_id: String) -> Self {
        self.suggested_checkpoint_id = Some(checkpoint_id);
        self
    }

    /// 设置创建者
    pub fn with_created_by(mut self, agent_id: AgentId) -> Self {
        self.created_by = Some(agent_id);
        self
    }

    /// 获取工单 ID
    pub fn id(&self) -> WorkOrderId {
        self.id
    }

    /// 检查是否是求助工单
    pub fn is_help_request(&self) -> bool {
        matches!(self.work_order_type, WorkOrderType::HelpRequest)
    }

    /// 检查是否是任务完成工单
    pub fn is_task_completion(&self) -> bool {
        matches!(self.work_order_type, WorkOrderType::TaskCompletion)
    }

    /// 检查是否是接力转交工单
    pub fn is_handover(&self) -> bool {
        matches!(self.work_order_type, WorkOrderType::Handover)
    }
}

impl fmt::Display for WorkOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WorkOrder[{}] {} -> {}: {}",
            self.id, self.work_order_type, self.recipient, self.title
        )
    }
}

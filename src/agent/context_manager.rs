//! 上下文管理器
//!
//! 提供 CMA 上下文管理功能。

use tracing::info;

use crate::agent::context::{ContextChunk, ContextChunkType, ContextStore};
use crate::agent::work_order::WorkOrder;
use crate::error::{Error, Result};
use crate::models::Session;
use crate::orchestrator::types::{CmaNotification, CmaNotificationType};

// ============================================================================
// ContextManagerAgent - 上下文管理 Agent
// ============================================================================

/// 上下文管理 Agent
///
/// 负责监控和维护所有会话的上下文，处理裁剪和求助请求。
pub struct ContextManagerAgent {
    /// 上下文存储
    pub store: ContextStore,
    /// 裁剪触发阈值（Token 数）
    pub global_max_tokens: usize,
    /// 裁剪后注入的提示词
    pub trim_hint: String,
    /// 裁剪阈值（默认 0.6，即 60% 时触发裁剪）
    pub threshold: f64,
}

impl ContextManagerAgent {
    /// 创建新的 ContextManagerAgent
    pub fn new(store: ContextStore, max_tokens: usize) -> Self {
        Self {
            store,
            global_max_tokens: max_tokens,
            trim_hint: "注意：之前的对话上下文已被 CMA 裁剪以保持注意力专注".to_string(),
            threshold: 0.6,
        }
    }

    /// 创建新的 ContextManagerAgent（完整配置）
    pub fn with_config(
        store: ContextStore,
        max_tokens: usize,
        trim_hint: String,
        threshold: f64,
    ) -> Self {
        Self {
            store,
            global_max_tokens: max_tokens,
            trim_hint,
            threshold,
        }
    }

    /// 分析内容复杂度并返回调整后的阈值
    ///
    /// 当检测到任务复杂度高时，降低阈值以保留更多上下文
    fn analyze_and_adjust_threshold(&self, chunks: &[ContextChunk]) -> f64 {
        if chunks.is_empty() {
            return self.threshold;
        }

        let mut complexity_score = 0.0;
        let mut help_request_count = 0;
        let mut tool_call_count = 0;

        for chunk in chunks {
            match chunk.chunk_type {
                ContextChunkType::HelpRequest => help_request_count += 1,
                ContextChunkType::ToolCall => tool_call_count += 1,
                _ => {}
            }

            if chunk.is_important {
                complexity_score += 1.0;
            }

            complexity_score += chunk.retention_priority as f64 / 10.0;
        }

        let chunk_count = chunks.len() as f64;
        complexity_score /= chunk_count.max(1.0);

        let mut adjusted_threshold = self.threshold;

        if help_request_count > 0 {
            adjusted_threshold = (adjusted_threshold + 0.15).min(0.9);
        }

        if tool_call_count > 3 {
            adjusted_threshold = (adjusted_threshold + 0.10).min(0.85);
        }

        if complexity_score > 0.7 {
            adjusted_threshold = (adjusted_threshold + 0.15).min(0.9);
        }

        adjusted_threshold
    }

    /// 向会话添加上下文并监控限制
    pub async fn add_chunk_and_monitor(
        &self,
        chunk: ContextChunk,
        session: &Session,
    ) -> Result<Option<CmaNotification>> {
        let session_id = chunk.session_id;
        self.store.add_chunk(chunk).await;

        let chunks = self.store.get_chunks(&session_id).await;
        let current_token_count = chunks.iter().map(|c| c.token_count).sum::<usize>();

        let trigger_threshold =
            (self.global_max_tokens as f64 * self.analyze_and_adjust_threshold(&chunks)) as usize;

        if current_token_count > trigger_threshold {
            info!(
                session_id = %session_id,
                current_tokens = %current_token_count,
                trigger_threshold = %trigger_threshold,
                "Context approaching limit, triggering auto-trim"
            );

            let target_tokens = (self.global_max_tokens as f64 * 0.8) as usize;
            let removed_count = current_token_count - target_tokens;

            let mut remaining = chunks;
            while remaining.iter().map(|c| c.token_count).sum::<usize>() > target_tokens
                && remaining.len() > 1
            {
                if remaining[0].is_important {
                    if let Some(pos) = remaining.iter().position(|c| !c.is_important) {
                        remaining.remove(pos);
                    } else {
                        break;
                    }
                } else {
                    remaining.remove(0);
                }
            }

            self.store
                .update_session_chunks(session_id, remaining.clone())
                .await;

            let trim_hint_chunk = ContextChunk::new(
                session_id,
                self.trim_hint.clone(),
                ContextChunkType::SystemNotification,
                self.trim_hint.len() / 4,
            );
            self.store.add_chunk(trim_hint_chunk).await;

            info!(
                session_id = %session_id,
                removed_tokens = %removed_count,
                "Context trimmed successfully"
            );

            if let Some(orchestrator_id) = session.orchestrator_id {
                return Ok(Some(CmaNotification::new(
                    CmaNotificationType::ContextTrimmed,
                    session_id,
                    orchestrator_id,
                    format!(
                        "Automatically trimmed {} tokens due to context limit",
                        removed_count
                    ),
                )));
            }
        }

        Ok(None)
    }

    /// 处理求助工单
    pub async fn handle_help_request(
        &self,
        work_order: WorkOrder,
        session: &Session,
    ) -> Result<CmaNotification> {
        let session_id = work_order.session_id;
        let orchestrator_id = session.orchestrator_id.ok_or_else(|| Error::Internal)?;

        info!(
            session_id = %session_id,
            work_order_id = %work_order.id(),
            "Processing help request in CMA"
        );

        self.store
            .add_chunk(ContextChunk::from_work_order(&work_order))
            .await;

        let mut notification = CmaNotification::new(
            CmaNotificationType::RollbackAndHandover,
            session_id,
            orchestrator_id,
            format!("Agent requested help: {}", work_order.title),
        );

        if let Some(checkpoint_id) = work_order.suggested_checkpoint_id {
            notification = notification.with_checkpoint_id(checkpoint_id);
        }

        Ok(notification)
    }
}

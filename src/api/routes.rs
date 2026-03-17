use axum::Router;
use axum::routing::{delete, get, post};

use crate::api::handlers;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/api/messages", post(handlers::send_message))
        .route("/api/messages/stream", post(handlers::send_message_stream))
        .route("/api/sessions", get(handlers::list_sessions))
        .route("/api/sessions/{id}", get(handlers::get_session))
        .route("/api/sessions/{id}", delete(handlers::delete_session))
        .route("/api/sessions/{id}/messages", get(handlers::list_messages))
        .route("/api/sessions/{id}/stream", get(handlers::session_stream))
        // 管理 API
        .route("/api/tools", get(handlers::list_tools))
        .route("/api/mcp/servers", get(handlers::list_mcp_servers))
        .route(
            "/api/mcp/servers/{name}/restart",
            post(handlers::restart_mcp_server),
        )
        // 调试 API
        .route("/api/debug/info", get(handlers::debug_info))
        .route("/api/debug/config", get(handlers::debug_config))
        .route("/api/debug/echo", post(handlers::debug_echo))
        .route("/api/debug/sessions/count", get(handlers::debug_session_count))
        // 终端测试 API
        .route("/api/debug/terminal/run", post(handlers::debug_terminal_run))
        .route(
            "/api/debug/terminal/test-output",
            post(handlers::debug_terminal_test_output),
        )
        .route(
            "/api/debug/terminal/test-timeout",
            post(handlers::debug_terminal_test_timeout),
        )
        .route(
            "/api/debug/terminal/test-truncation",
            post(handlers::debug_terminal_test_truncation),
        )
        .with_state(state)
}

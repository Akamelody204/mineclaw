use std::env;
use std::io::{self, Write};
use tokio::process::Command;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

// 引入 mineclaw 库中的协议定义
use mineclaw::mcp::protocol::{
    JsonRpcRequest, JsonRpcResponse, JsonRpcError,
    InitializeResponse, ServerCapabilities, Implementation,
    ListToolsResponse, ProtocolTool,
    CallToolRequest, CallToolResponse, ToolResultContent,
    ToolsCapability
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志到 stderr
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    let mut shell_path: Option<String> = None;
    
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--shell" && i + 1 < args.len() {
            shell_path = Some(args[i+1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }

    let shell_type = detect_shell_type(shell_path.as_deref());
    info!("Starting Terminal MCP Server (Rust). Shell: {:?} Type: {}", shell_path, shell_type);

    let (tx, mut rx) = mpsc::channel::<JsonRpcResponse>(100);

    // 启动输出处理任务
    tokio::spawn(async move {
        let mut stdout = io::stdout();
        while let Some(response) = rx.recv().await {
            match serde_json::to_string(&response) {
                Ok(json) => {
                    let _ = writeln!(stdout, "{}", json);
                    let _ = stdout.flush();
                }
                Err(e) => {
                    error!("Failed to serialize response: {}", e);
                }
            }
        }
    });

    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                let tx = tx.clone();
                let shell_path = shell_path.clone();
                let shell_type = shell_type.clone();
                
                tokio::spawn(async move {
                    handle_request(request, tx, shell_path, shell_type).await;
                });
            }
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
            }
        }
    }

    Ok(())
}

fn detect_shell_type(shell_path: Option<&str>) -> String {
    if let Some(path) = shell_path {
        let lower = path.to_lowercase();
        if lower.contains("powershell") || lower.contains("pwsh") {
            return "powershell".to_string();
        }
        if lower.contains("bash") {
            return "bash".to_string();
        }
        if lower.contains("zsh") {
            return "zsh".to_string();
        }
        if lower.contains("cmd.exe") {
            return "cmd".to_string();
        }
    }
    
    if cfg!(windows) {
        "cmd".to_string()
    } else {
        "bash".to_string()
    }
}

async fn handle_request(
    request: JsonRpcRequest,
    tx: mpsc::Sender<JsonRpcResponse>,
    shell_path: Option<String>,
    shell_type: String,
) {
    let id = request.id.clone();
    let method = request.method.as_str();

    let result = match method {
        "initialize" => handle_initialize().await,
        "notifications/initialized" => Ok(None),
        "tools/list" => handle_list_tools(&shell_type).await,
        "tools/call" => handle_call_tool(request, shell_path).await,
        _ => {
             Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", method),
                data: None,
            })
        }
    };

    let response = match result {
        Ok(Some(res_value)) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(res_value),
            error: None,
        },
        Ok(None) => return, // Notification or handled
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(e),
        },
    };

    if let Err(e) = tx.send(response).await {
        error!("Failed to send response: {}", e);
    }
}

async fn handle_initialize() -> Result<Option<Value>, JsonRpcError> {
    let response = InitializeResponse {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities {
            tools: ToolsCapability {
                list_changed: false,
            },
            ..Default::default()
        },
        server_info: Implementation {
            name: "terminal-server-rust".to_string(),
            version: "1.0.0".to_string(),
        },
    };
    Ok(Some(serde_json::to_value(response).unwrap()))
}

async fn handle_list_tools(shell_type: &str) -> Result<Option<Value>, JsonRpcError> {
    let tools = vec![
        ProtocolTool {
            name: "execute_command".to_string(),
            description: format!("Execute a command in the terminal (Current Shell: {})", shell_type),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The command to execute"
                    }
                },
                "required": ["command"]
            }),
        },
        ProtocolTool {
            name: "get_shell_info".to_string(),
            description: "Get information about the current shell environment".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
    ];

    let response = ListToolsResponse {
        tools,
        next_cursor: None,
    };
    Ok(Some(serde_json::to_value(response).unwrap()))
}

async fn handle_call_tool(request: JsonRpcRequest, shell_path: Option<String>) -> Result<Option<Value>, JsonRpcError> {
    let params = request.params.ok_or(JsonRpcError {
        code: -32602,
        message: "Missing params".to_string(),
        data: None,
    })?;

    let call_req: CallToolRequest = serde_json::from_value(params).map_err(|e| JsonRpcError {
        code: -32602,
        message: format!("Invalid params: {}", e),
        data: None,
    })?;

    match call_req.name.as_str() {
        "execute_command" => {
            let command_str = call_req.arguments.get("command")
                .and_then(|v| v.as_str())
                .ok_or(JsonRpcError {
                    code: -32602,
                    message: "Missing 'command' argument".to_string(),
                    data: None,
                })?;

            info!("Executing command: {}", command_str);

            let (sh_prog, sh_arg) = if let Some(sh) = &shell_path {
                if sh.to_lowercase().contains("cmd.exe") {
                    (sh.clone(), "/C")
                } else if sh.to_lowercase().contains("powershell") || sh.to_lowercase().contains("pwsh") {
                     (sh.clone(), "-Command")
                } else {
                     (sh.clone(), "-c")
                }
            } else {
                 if cfg!(windows) {
                    ("cmd".to_string(), "/C")
                 } else {
                    ("sh".to_string(), "-c")
                 }
            };

            let output = Command::new(sh_prog)
                .arg(sh_arg)
                .arg(command_str)
                .output()
                .await;

            match output {
                Ok(output) => {
                    let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let mut stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    
                    const MAX_OUTPUT_LEN: usize = 32 * 1024; // 32KB
                    
                    if stdout.len() > MAX_OUTPUT_LEN {
                        stdout.truncate(MAX_OUTPUT_LEN);
                        stdout.push_str("\n\n[System Warning] Output truncated (exceeded 32KB). Please use tools like `head`, `tail`, or `grep` to view specific parts.");
                    }
                    
                    if stderr.len() > MAX_OUTPUT_LEN {
                        stderr.truncate(MAX_OUTPUT_LEN);
                        stderr.push_str("\n\n[System Warning] Stderr truncated (exceeded 32KB).");
                    }
                    
                    let mut text = stdout;
                    if !stderr.is_empty() {
                        text.push_str("\nStderr:\n");
                        text.push_str(&stderr);
                    }
                    
                    let response = CallToolResponse {
                        content: vec![ToolResultContent::Text { text }],
                        is_error: !output.status.success(),
                    };
                    Ok(Some(serde_json::to_value(response).unwrap()))
                }
                Err(e) => {
                    let response = CallToolResponse {
                        content: vec![ToolResultContent::Text { text: format!("Execution failed: {}", e) }],
                        is_error: true,
                    };
                    Ok(Some(serde_json::to_value(response).unwrap()))
                }
            }
        }
        "get_shell_info" => {
            let info = format!(
                "Shell Path: {:?}\nPlatform: {}",
                shell_path,
                env::consts::OS
            );
             let response = CallToolResponse {
                content: vec![ToolResultContent::Text { text: info }],
                is_error: false,
            };
            Ok(Some(serde_json::to_value(response).unwrap()))
        }
        _ => Err(JsonRpcError {
            code: -32601,
            message: format!("Tool not found: {}", call_req.name),
            data: None,
        }),
    }
}

//! 文件系统工具实现
//!
//! 提供读写文件、列目录、搜索等基础文件系统操作。

use crate::error::{Error, Result};
use crate::tools::{LocalTool, ToolContext};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ==================== 辅助函数 ====================

/// 规范化并验证路径是否在允许的目录范围内
fn normalize_and_validate_path(raw_path: &str, allowed_dirs: &[String]) -> Result<PathBuf> {
    let path = Path::new(raw_path);

    // 处理相对于当前目录的路径，或者已经是绝对路径的情况
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    // 规范化路径，处理 .. 和 .
    let normalized = if abs_path.exists() {
        abs_path.canonicalize()?
    } else if let Some(parent) = abs_path.parent() {
        if parent.exists() {
            parent
                .canonicalize()?
                .join(abs_path.file_name().unwrap_or_default())
        } else {
            abs_path // 无法规范化，保持原样
        }
    } else {
        abs_path
    };

    // 检查是否在允许的目录中
    if allowed_dirs.is_empty() {
        return Ok(normalized);
    }

    let strip_unc = |p: PathBuf| -> String {
        let s = p.to_string_lossy().to_string();
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            stripped.to_string()
        } else {
            s
        }
    };

    let clean_normalized = strip_unc(normalized.clone());

    for allowed_dir in allowed_dirs {
        let clean_allowed = if let Ok(allowed_path) = Path::new(allowed_dir).canonicalize() {
            strip_unc(allowed_path)
        } else {
            allowed_dir.to_string()
        };

        let matches = if cfg!(windows) {
            clean_normalized
                .to_lowercase()
                .starts_with(&clean_allowed.to_lowercase())
        } else {
            clean_normalized.starts_with(&clean_allowed)
        };

        if matches {
            return Ok(normalized);
        }
    }

    Err(Error::LocalToolExecution {
        tool: "filesystem".to_string(),
        message: format!("Path is not allowed: {}", raw_path),
    })
}

fn get_filesystem_config(context: &ToolContext) -> crate::config::FilesystemConfig {
    context.config.filesystem.clone()
}

/// 如果启用了 checkpoint，则创建 checkpoint
async fn maybe_create_checkpoint(
    context: &ToolContext,
    affected_files: Vec<String>,
    description: Option<String>,
) -> Result<()> {
    if let Some(cm) = &context.checkpoint_manager {
        cm.create_checkpoint(
            &context.session,
            description,
            crate::models::CheckpointType::Manual,
            Some(affected_files),
        )
        .await?;
    }
    Ok(())
}

// ==================== 参数和结果定义 ====================

#[derive(Debug, Deserialize)]
pub struct ReadFileParams {
    pub path: String,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ReadFileResult {
    pub content: String,
    pub truncated: bool,
    pub total_bytes: usize,
}

#[derive(Debug, Deserialize)]
pub struct WriteFileParams {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct WriteFileResult {
    pub success: bool,
    pub bytes_written: usize,
}

#[derive(Debug, Deserialize)]
pub struct ListDirectoryParams {
    pub path: String,
    pub recursive: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Serialize)]
pub struct ListDirectoryResult {
    pub entries: Vec<DirectoryEntry>,
}

#[derive(Debug, Deserialize)]
pub struct DeletePathParams {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct GrepParams {
    pub regex: String,
    pub include_pattern: String,
    pub case_sensitive: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SearchMatch {
    pub line_number: usize,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct GrepResult {
    pub matches: Vec<SearchMatch>,
}

#[derive(Debug, Deserialize)]
pub struct MovePathParams {
    pub source_path: String,
    pub destination_path: String,
}

#[derive(Debug, Deserialize)]
pub struct CopyPathParams {
    pub source_path: String,
    pub destination_path: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDirectoryParams {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchAndReplaceParams {
    pub path: String,
    pub diff: String,
    pub global: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SearchAndReplaceResult {
    pub success: bool,
    pub replacements: usize,
}

#[derive(Debug, Deserialize)]
pub struct ReplaceAllKeywordsParams {
    pub path: String,
    pub search: String,
    pub replace: String,
    pub case_sensitive: Option<bool>,
    pub use_regex: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ReplaceAllKeywordsResult {
    pub success: bool,
    pub replacements: usize,
}

// ==================== 工具实现 ====================

struct ReadFileTool;

#[async_trait]
impl LocalTool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file, with optional line range and truncation"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "start_line": { "type": "integer" },
                "end_line": { "type": "integer" }
            },
            "required": ["path"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let params: ReadFileParams = serde_json::from_value(arguments)?;
        let config = get_filesystem_config(&context);
        let path = normalize_and_validate_path(&params.path, &config.allowed_directories)?;

        let full_content = std::fs::read_to_string(&path)?;

        // 处理行范围
        let content = if params.start_line.is_some() || params.end_line.is_some() {
            let start = params.start_line.unwrap_or(1).saturating_sub(1);
            let end = params.end_line.unwrap_or(usize::MAX);
            full_content
                .lines()
                .enumerate()
                .filter(|(i, _)| *i >= start && *i < end)
                .map(|(_, l)| l)
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            full_content
        };

        let total_bytes = content.len();
        let mut truncated = false;
        let final_content = if total_bytes > config.max_read_bytes {
            truncated = true;
            let mut end = config.max_read_bytes;
            while end > 0 && !content.is_char_boundary(end) {
                end -= 1;
            }
            content[..end].to_string()
        } else {
            content
        };

        Ok(json!(ReadFileResult {
            content: final_content,
            truncated,
            total_bytes,
        }))
    }
}

struct WriteFileTool;

#[async_trait]
impl LocalTool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let params: WriteFileParams = serde_json::from_value(arguments)?;
        let config = get_filesystem_config(&context);
        let path = normalize_and_validate_path(&params.path, &config.allowed_directories)?;

        let _ = maybe_create_checkpoint(
            &context,
            vec![params.path.clone()],
            Some(format!("Before writing to {}", params.path)),
        )
        .await;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&path, &params.content)?;

        Ok(json!(WriteFileResult {
            success: true,
            bytes_written: params.content.len(),
        }))
    }
}

struct ListDirectoryTool;

#[async_trait]
impl LocalTool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List contents of a directory"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "recursive": { "type": "boolean" }
            },
            "required": ["path"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let params: ListDirectoryParams = serde_json::from_value(arguments)?;
        let config = get_filesystem_config(&context);
        let abs_dir = normalize_and_validate_path(&params.path, &config.allowed_directories)?;

        let mut entries = Vec::new();
        if params.recursive.unwrap_or(false) {
            for entry in walkdir::WalkDir::new(&abs_dir).min_depth(1) {
                let entry = entry?;
                let rel_path = entry.path().strip_prefix(&abs_dir).unwrap_or(entry.path());
                let display_path = Path::new(&params.path).join(rel_path);
                entries.push(DirectoryEntry {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: display_path.to_string_lossy().to_string(),
                    is_dir: entry.file_type().is_dir(),
                });
            }
        } else {
            for entry in std::fs::read_dir(&abs_dir)? {
                let entry = entry?;
                let file_name = entry.file_name();
                let display_path = Path::new(&params.path).join(&file_name);
                entries.push(DirectoryEntry {
                    name: file_name.to_string_lossy().to_string(),
                    path: display_path.to_string_lossy().to_string(),
                    is_dir: entry.file_type()?.is_dir(),
                });
            }
        }

        Ok(json!(ListDirectoryResult { entries }))
    }
}

struct DeletePathTool;

#[async_trait]
impl LocalTool for DeletePathTool {
    fn name(&self) -> &str {
        "delete_path"
    }

    fn description(&self) -> &str {
        "Delete a file or directory"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "path": { "type": "string" } },
            "required": ["path"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let params: DeletePathParams = serde_json::from_value(arguments)?;
        let config = get_filesystem_config(&context);
        let path = normalize_and_validate_path(&params.path, &config.allowed_directories)?;

        let _ = maybe_create_checkpoint(
            &context,
            vec![params.path.clone()],
            Some(format!("Before deleting {}", params.path)),
        )
        .await;

        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else {
            std::fs::remove_file(&path)?;
        }

        Ok(json!({ "success": true }))
    }
}

struct GrepTool;

#[async_trait]
impl LocalTool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search for a regex pattern in a file"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "regex": { "type": "string" },
                "include_pattern": { "type": "string" },
                "case_sensitive": { "type": "boolean" }
            },
            "required": ["regex", "include_pattern"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let params: GrepParams = serde_json::from_value(arguments)?;
        let config = get_filesystem_config(&context);
        let path =
            normalize_and_validate_path(&params.include_pattern, &config.allowed_directories)?;

        let content = std::fs::read_to_string(&path)?;
        let re = if params.case_sensitive.unwrap_or(false) {
            regex::Regex::new(&params.regex)?
        } else {
            regex::Regex::new(&format!("(?i){}", params.regex))?
        };

        let matches: Vec<SearchMatch> = content
            .lines()
            .enumerate()
            .filter(|(_, line)| re.is_match(line))
            .map(|(i, line)| SearchMatch {
                line_number: i + 1,
                content: line.to_string(),
            })
            .collect();

        Ok(json!(GrepResult { matches }))
    }
}

struct MovePathTool;

#[async_trait]
impl LocalTool for MovePathTool {
    fn name(&self) -> &str {
        "move_path"
    }

    fn description(&self) -> &str {
        "Move or rename a file or directory"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "source_path": { "type": "string" },
                "destination_path": { "type": "string" }
            },
            "required": ["source_path", "destination_path"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let params: MovePathParams = serde_json::from_value(arguments)?;
        let config = get_filesystem_config(&context);
        let src = normalize_and_validate_path(&params.source_path, &config.allowed_directories)?;
        let dst =
            normalize_and_validate_path(&params.destination_path, &config.allowed_directories)?;

        let _ = maybe_create_checkpoint(
            &context,
            vec![params.source_path.clone(), params.destination_path.clone()],
            Some(format!(
                "Move {} to {}",
                params.source_path, params.destination_path
            )),
        )
        .await;

        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(src, dst)?;

        Ok(json!({ "success": true }))
    }
}

struct CopyPathTool;

#[async_trait]
impl LocalTool for CopyPathTool {
    fn name(&self) -> &str {
        "copy_path"
    }

    fn description(&self) -> &str {
        "Copy a file or directory"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "source_path": { "type": "string" },
                "destination_path": { "type": "string" }
            },
            "required": ["source_path", "destination_path"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let params: CopyPathParams = serde_json::from_value(arguments)?;
        let config = get_filesystem_config(&context);
        let src = normalize_and_validate_path(&params.source_path, &config.allowed_directories)?;
        let dst =
            normalize_and_validate_path(&params.destination_path, &config.allowed_directories)?;

        if src.is_dir() {
            copy_dir_recursive(&src, &dst)?;
        } else {
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(src, dst)?;
        }

        Ok(json!({ "success": true }))
    }
}

/// 辅助函数：递归复制目录
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

struct CreateDirectoryTool;

#[async_trait]
impl LocalTool for CreateDirectoryTool {
    fn name(&self) -> &str {
        "create_directory"
    }

    fn description(&self) -> &str {
        "Create a new directory"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "path": { "type": "string" } },
            "required": ["path"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let params: CreateDirectoryParams = serde_json::from_value(arguments)?;
        let config = get_filesystem_config(&context);
        let path = normalize_and_validate_path(&params.path, &config.allowed_directories)?;

        std::fs::create_dir_all(path)?;
        Ok(json!({ "success": true }))
    }
}

struct FindPathTool;

#[async_trait]
impl LocalTool for FindPathTool {
    fn name(&self) -> &str {
        "find_path"
    }

    fn description(&self) -> &str {
        "Find files by glob pattern"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "glob": { "type": "string" } },
            "required": ["glob"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let glob = arguments["glob"]
            .as_str()
            .ok_or_else(|| Error::LocalToolExecution {
                tool: "find_path".to_string(),
                message: "Missing glob parameter".to_string(),
            })?;
        let config = get_filesystem_config(&context);

        let mut matches = Vec::new();
        for allowed_dir in &config.allowed_directories {
            let root = Path::new(allowed_dir);
            if !root.exists() {
                continue;
            }

            for entry in walkdir::WalkDir::new(root)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path_str = entry.path().to_string_lossy();
                // 极简实现：仅检查后缀或包含
                if let Some(ext) = glob.strip_prefix("**/*") {
                    if path_str.ends_with(ext) {
                        matches.push(path_str.to_string());
                    }
                } else if path_str.contains(glob) {
                    matches.push(path_str.to_string());
                }
            }
        }

        Ok(json!({ "matches": matches }))
    }
}

struct SearchAndReplaceTool;

pub fn parse_search_replace_blocks(diff: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut lines = diff.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim() == "------- SEARCH" {
            let mut search = Vec::new();
            let mut replace = Vec::new();
            for l in lines.by_ref() {
                if l.trim() == "=======" {
                    break;
                }
                search.push(l);
            }
            for l in lines.by_ref() {
                if l.trim() == "+++++++ REPLACE" {
                    break;
                }
                replace.push(l);
            }
            blocks.push((search.join("\n"), replace.join("\n")));
        }
    }
    blocks
}

#[async_trait]
impl LocalTool for SearchAndReplaceTool {
    fn name(&self) -> &str {
        "search_and_replace"
    }

    fn description(&self) -> &str {
        "Replace blocks of text in a file"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "diff": { "type": "string" },
                "global": { "type": "boolean" }
            },
            "required": ["path", "diff"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let params: SearchAndReplaceParams = serde_json::from_value(arguments)?;
        let config = get_filesystem_config(&context);
        let path = normalize_and_validate_path(&params.path, &config.allowed_directories)?;

        let _ = maybe_create_checkpoint(
            &context,
            vec![params.path.clone()],
            Some(format!("Search/Replace in {}", params.path)),
        )
        .await;

        let mut content = std::fs::read_to_string(&path)?;
        let blocks = parse_search_replace_blocks(&params.diff);
        let mut replacements = 0;

        for (search, replace) in blocks {
            if params.global.unwrap_or(true) {
                replacements += content.matches(&search).count();
                content = content.replace(&search, &replace);
            } else if let Some(pos) = content.find(&search) {
                content.replace_range(pos..pos + search.len(), &replace);
                replacements += 1;
            }
        }

        std::fs::write(&path, content)?;
        Ok(json!(SearchAndReplaceResult {
            success: true,
            replacements
        }))
    }
}

struct ReplaceAllKeywordsTool;

#[async_trait]
impl LocalTool for ReplaceAllKeywordsTool {
    fn name(&self) -> &str {
        "replace_all_keywords"
    }

    fn description(&self) -> &str {
        "Replace all occurrences of a keyword or pattern"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "search": { "type": "string" },
                "replace": { "type": "string" },
                "case_sensitive": { "type": "boolean" },
                "use_regex": { "type": "boolean" }
            },
            "required": ["path", "search", "replace"]
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let params: ReplaceAllKeywordsParams = serde_json::from_value(arguments)?;
        let config = get_filesystem_config(&context);
        let path = normalize_and_validate_path(&params.path, &config.allowed_directories)?;

        let _ = maybe_create_checkpoint(
            &context,
            vec![params.path.clone()],
            Some(format!("Replace keyword in {}", params.path)),
        )
        .await;

        let mut content = std::fs::read_to_string(&path)?;
        let replacements;

        if params.use_regex.unwrap_or(false) {
            let re = if params.case_sensitive.unwrap_or(true) {
                regex::Regex::new(&params.search)?
            } else {
                regex::Regex::new(&format!("(?i){}", params.search))?
            };
            replacements = re.find_iter(&content).count();
            content = re
                .replace_all(&content, params.replace.as_str())
                .to_string();
        } else {
            if params.case_sensitive.unwrap_or(true) {
                replacements = content.matches(&params.search).count();
                content = content.replace(&params.search, &params.replace);
            } else {
                let re = regex::Regex::new(&format!("(?i){}", regex::escape(&params.search)))?;
                replacements = re.find_iter(&content).count();
                content = re
                    .replace_all(&content, params.replace.as_str())
                    .to_string();
            }
        }

        std::fs::write(&path, content)?;
        Ok(json!(ReplaceAllKeywordsResult {
            success: true,
            replacements
        }))
    }
}

pub struct FilesystemTool;

impl FilesystemTool {
    pub fn register_all(registry: &mut super::registry::LocalToolRegistry) {
        registry.register(Arc::new(ReadFileTool));
        registry.register(Arc::new(WriteFileTool));
        registry.register(Arc::new(ListDirectoryTool));
        registry.register(Arc::new(DeletePathTool));
        registry.register(Arc::new(GrepTool));
        registry.register(Arc::new(MovePathTool));
        registry.register(Arc::new(CopyPathTool));
        registry.register(Arc::new(CreateDirectoryTool));
        registry.register(Arc::new(FindPathTool));
        registry.register(Arc::new(SearchAndReplaceTool));
        registry.register(Arc::new(ReplaceAllKeywordsTool));
    }
}

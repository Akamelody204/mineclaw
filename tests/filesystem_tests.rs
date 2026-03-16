//! 文件系统工具测试

use mineclaw::config::{Config, FilesystemConfig};
use mineclaw::error::Error;
use mineclaw::models::Session;
use mineclaw::tools::filesystem::FilesystemTool;
use mineclaw::tools::{LocalToolRegistry, ToolContext};
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;

/// 创建测试用的 ToolContext
fn create_test_context(config: Config) -> ToolContext {
    ToolContext::new(Session::new(), Arc::new(config))
}

#[tokio::test]
async fn test_read_write_file() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    let config = Config {
        filesystem: FilesystemConfig {
            max_read_bytes: 16384,
            allowed_directories: vec![temp_dir.path().to_string_lossy().to_string()],
        },
        ..Config::default()
    };

    let mut registry = LocalToolRegistry::new();
    FilesystemTool::register_all(&mut registry);

    let context = create_test_context(config);

    // Write file
    let write_result = registry
        .call_tool(
            "write_file",
            json!({
                "path": test_file.to_string_lossy().to_string(),
                "content": "Hello, World!"
            }),
            context.clone(),
        )
        .await
        .unwrap();

    assert!(write_result["success"].as_bool().unwrap());
    assert_eq!(write_result["bytes_written"].as_u64().unwrap(), 13);

    // Read file
    let read_result = registry
        .call_tool(
            "read_file",
            json!({
                "path": test_file.to_string_lossy().to_string()
            }),
            context,
        )
        .await
        .unwrap();

    assert_eq!(read_result["content"].as_str().unwrap(), "Hello, World!");
    assert!(!read_result["truncated"].as_bool().unwrap());
}

#[tokio::test]
async fn test_list_directory() {
    // Test with absolute path and relative path
    let temp_dir = tempdir().unwrap();
    let temp_dir_abs = temp_dir.path().canonicalize().unwrap();
    let temp_dir_str = temp_dir_abs.to_string_lossy().to_string();

    // Create test files
    std::fs::File::create(temp_dir_abs.join("file1.txt")).unwrap();
    std::fs::File::create(temp_dir_abs.join("file2.txt")).unwrap();
    std::fs::create_dir(temp_dir_abs.join("subdir")).unwrap();
    std::fs::File::create(temp_dir_abs.join("subdir/nested.txt")).unwrap();

    // Change to temp dir's parent to test relative paths
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir_abs.parent().unwrap()).unwrap();

    let config = Config {
        filesystem: FilesystemConfig {
            max_read_bytes: 16384,
            allowed_directories: vec![temp_dir_str.clone()],
        },
        ..Config::default()
    };

    let mut registry = LocalToolRegistry::new();
    FilesystemTool::register_all(&mut registry);

    let context = create_test_context(config);

    // Test 1: List with relative path
    let temp_dir_name = temp_dir_abs.file_name().unwrap().to_string_lossy();
    let result = registry
        .call_tool(
            "list_directory",
            json!({
                "path": temp_dir_name.to_string()
            }),
            context.clone(),
        )
        .await
        .unwrap();

    let entries = result["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);

    let names: Vec<&str> = entries
        .iter()
        .map(|i| i["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"file1.txt"));
    assert!(names.contains(&"file2.txt"));
    assert!(names.contains(&"subdir"));

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_create_directory() {
    let temp_dir = tempdir().unwrap();
    let new_dir = temp_dir.path().join("new_dir");

    let config = Config {
        filesystem: FilesystemConfig {
            max_read_bytes: 16384,
            allowed_directories: vec![temp_dir.path().to_string_lossy().to_string()],
        },
        ..Config::default()
    };

    let mut registry = LocalToolRegistry::new();
    FilesystemTool::register_all(&mut registry);

    let context = create_test_context(config);

    let result = registry
        .call_tool(
            "create_directory",
            json!({
                "path": new_dir.to_string_lossy().to_string()
            }),
            context,
        )
        .await
        .unwrap();

    assert!(result["success"].as_bool().unwrap());
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
}

#[tokio::test]
async fn test_move_path() {
    let temp_dir = tempdir().unwrap();
    let source = temp_dir.path().join("source.txt");
    let dest = temp_dir.path().join("dest.txt");

    std::fs::write(&source, "Hello").unwrap();

    let config = Config {
        filesystem: FilesystemConfig {
            max_read_bytes: 16384,
            allowed_directories: vec![temp_dir.path().to_string_lossy().to_string()],
        },
        ..Config::default()
    };

    let mut registry = LocalToolRegistry::new();
    FilesystemTool::register_all(&mut registry);

    let context = create_test_context(config);

    let result = registry
        .call_tool(
            "move_path",
            json!({
                "source_path": source.to_string_lossy().to_string(),
                "destination_path": dest.to_string_lossy().to_string()
            }),
            context,
        )
        .await
        .unwrap();

    assert!(result["success"].as_bool().unwrap());
    assert!(!source.exists());
    assert!(dest.exists());
    assert_eq!(std::fs::read_to_string(dest).unwrap(), "Hello");
}

#[tokio::test]
async fn test_copy_path() {
    let temp_dir = tempdir().unwrap();
    let source = temp_dir.path().join("source.txt");
    let dest = temp_dir.path().join("dest.txt");

    std::fs::write(&source, "Hello").unwrap();

    let config = Config {
        filesystem: FilesystemConfig {
            max_read_bytes: 16384,
            allowed_directories: vec![temp_dir.path().to_string_lossy().to_string()],
        },
        ..Config::default()
    };

    let mut registry = LocalToolRegistry::new();
    FilesystemTool::register_all(&mut registry);

    let context = create_test_context(config);

    let result = registry
        .call_tool(
            "copy_path",
            json!({
                "source_path": source.to_string_lossy().to_string(),
                "destination_path": dest.to_string_lossy().to_string()
            }),
            context,
        )
        .await
        .unwrap();

    assert!(result["success"].as_bool().unwrap());
    assert!(source.exists());
    assert!(dest.exists());
    assert_eq!(std::fs::read_to_string(dest).unwrap(), "Hello");
}

#[tokio::test]
async fn test_delete_path() {
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().join("target.txt");

    std::fs::write(&target, "Hello").unwrap();

    let config = Config {
        filesystem: FilesystemConfig {
            max_read_bytes: 16384,
            allowed_directories: vec![temp_dir.path().to_string_lossy().to_string()],
        },
        ..Config::default()
    };

    let mut registry = LocalToolRegistry::new();
    FilesystemTool::register_all(&mut registry);

    let context = create_test_context(config);

    let result = registry
        .call_tool(
            "delete_path",
            json!({
                "path": target.to_string_lossy().to_string()
            }),
            context,
        )
        .await
        .unwrap();

    assert!(result["success"].as_bool().unwrap());
    assert!(!target.exists());
}

#[tokio::test]
async fn test_path_security_restriction() {
    let temp_dir = tempdir().unwrap();
    let outside_dir = tempdir().unwrap();
    let outside_file = outside_dir.path().join("secret.txt");
    std::fs::write(&outside_file, "secret").unwrap();

    let config = Config {
        filesystem: FilesystemConfig {
            max_read_bytes: 16384,
            allowed_directories: vec![temp_dir.path().to_string_lossy().to_string()],
        },
        ..Config::default()
    };

    let mut registry = LocalToolRegistry::new();
    FilesystemTool::register_all(&mut registry);

    let context = create_test_context(config);

    // Try to read file outside allowed directory
    let result = registry
        .call_tool(
            "read_file",
            json!({
                "path": outside_file.to_string_lossy().to_string()
            }),
            context,
        )
        .await;

    assert!(result.is_err());
    match result.err().unwrap() {
        Error::LocalToolExecution { message, .. } => {
            assert!(message.contains("not allowed"));
        }
        _ => panic!("Expected LocalToolExecution error"),
    }
}

#[tokio::test]
async fn test_read_file_with_line_numbers() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("lines.txt");
    let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
    std::fs::write(&test_file, content).unwrap();

    let config = Config {
        filesystem: FilesystemConfig {
            max_read_bytes: 16384,
            allowed_directories: vec![temp_dir.path().to_string_lossy().to_string()],
        },
        ..Config::default()
    };

    let mut registry = LocalToolRegistry::new();
    FilesystemTool::register_all(&mut registry);

    let context = create_test_context(config);

    // Read specific lines
    let result = registry
        .call_tool(
            "read_file",
            json!({
                "path": test_file.to_string_lossy().to_string(),
                "start_line": 2,
                "end_line": 4
            }),
            context,
        )
        .await
        .unwrap();

    let result_content = result["content"].as_str().unwrap();
    assert_eq!(result_content, "Line 2\nLine 3\nLine 4");
}

#[tokio::test]
async fn test_search_file_content() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("search.txt");
    let content = "The quick brown fox\njumps over the lazy dog\nFoxes are clever";
    std::fs::write(&test_file, content).unwrap();

    let config = Config {
        filesystem: FilesystemConfig {
            max_read_bytes: 16384,
            allowed_directories: vec![temp_dir.path().to_string_lossy().to_string()],
        },
        ..Config::default()
    };

    let mut registry = LocalToolRegistry::new();
    FilesystemTool::register_all(&mut registry);

    let context = create_test_context(config);

    // Search for "fox" (case-insensitive)
    let result = registry
        .call_tool(
            "grep",
            json!({
                "regex": "fox",
                "include_pattern": test_file.to_string_lossy().to_string(),
                "case_sensitive": false
            }),
            context,
        )
        .await
        .unwrap();

    let matches = result["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 2);

    let lines: Vec<u64> = matches
        .iter()
        .map(|m| m["line_number"].as_u64().unwrap())
        .collect();
    assert!(lines.contains(&1));
    assert!(lines.contains(&3));
}

#[tokio::test]
async fn test_find_files_by_pattern() {
    let temp_dir = tempdir().unwrap();
    std::fs::File::create(temp_dir.path().join("file1.rs")).unwrap();
    std::fs::File::create(temp_dir.path().join("file2.rs")).unwrap();
    std::fs::File::create(temp_dir.path().join("notes.txt")).unwrap();

    let config = Config {
        filesystem: FilesystemConfig {
            max_read_bytes: 16384,
            allowed_directories: vec![temp_dir.path().to_string_lossy().to_string()],
        },
        ..Config::default()
    };

    let mut registry = LocalToolRegistry::new();
    FilesystemTool::register_all(&mut registry);

    let context = create_test_context(config);

    // Find all .rs files
    let result = registry
        .call_tool(
            "find_path",
            json!({
                "glob": "**/*.rs"
            }),
            context,
        )
        .await
        .unwrap();

    let files = result["matches"].as_array().unwrap();
    assert_eq!(files.len(), 2);

    let names: Vec<String> = files
        .iter()
        .map(|f| f.as_str().unwrap().to_string())
        .collect();
    assert!(names.iter().any(|n| n.ends_with("file1.rs")));
    assert!(names.iter().any(|n| n.ends_with("file2.rs")));
}

#[tokio::test]
async fn test_read_file_size_limit() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("large.txt");
    // Create a file larger than max_read_bytes if possible, or just test limit
    let content = "A".repeat(100);

    std::fs::write(&test_file, &content).unwrap();

    let config = Config {
        filesystem: FilesystemConfig {
            max_read_bytes: 10, // Small limit for testing
            allowed_directories: vec![temp_dir.path().to_string_lossy().to_string()],
        },
        ..Config::default()
    };

    let mut registry = LocalToolRegistry::new();
    FilesystemTool::register_all(&mut registry);

    let context = create_test_context(config);

    let result = registry
        .call_tool(
            "read_file",
            json!({
                "path": test_file.to_string_lossy().to_string()
            }),
            context,
        )
        .await
        .unwrap();

    assert_eq!(result["content"].as_str().unwrap().len(), 10);
    assert!(result["truncated"].as_bool().unwrap());
}

#[tokio::test]
async fn test_path_traversal_prevention() {
    let temp_dir = tempdir().unwrap();
    let allowed_path = temp_dir.path().join("allowed");
    std::fs::create_dir(&allowed_path).unwrap();

    let config = Config {
        filesystem: FilesystemConfig {
            max_read_bytes: 16384,
            allowed_directories: vec![allowed_path.to_string_lossy().to_string()],
        },
        ..Config::default()
    };

    let mut registry = LocalToolRegistry::new();
    FilesystemTool::register_all(&mut registry);

    let context = create_test_context(config);

    // Try path traversal: allowed/../secret.txt
    let traversal_path = allowed_path.join("../secret.txt");

    let result = registry
        .call_tool(
            "read_file",
            json!({
                "path": traversal_path.to_string_lossy().to_string()
            }),
            context,
        )
        .await;

    assert!(result.is_err());
    match result.err().unwrap() {
        Error::LocalToolExecution { message, .. } => {
            assert!(message.contains("not allowed"));
        }
        _ => panic!("Expected LocalToolExecution error"),
    }
}

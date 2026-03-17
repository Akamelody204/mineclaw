//! Shell 检测模块
//! 提供检测当前 Shell 类型的功能

use serde::{Deserialize, Serialize};
use std::env;

/// Shell 类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellType {
    /// Windows cmd.exe
    Cmd,
    /// Windows PowerShell
    PowerShell,
    /// Git Bash (MinGW/MSYS2)
    GitBash,
    /// Windows Subsystem for Linux
    Wsl,
    /// Bourne Again Shell (bash)
    Bash,
    /// Z Shell (zsh)
    Zsh,
    /// Friendly Interactive Shell (fish)
    Fish,
    /// Bourne Shell (sh)
    Sh,
    /// 未知 Shell
    Unknown,
}

/// 操作系统类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatingSystem {
    Windows,
    Linux,
    MacOS,
    Other,
}

/// 系统信息（包含 OS 和 Shell）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// 操作系统类型
    pub os: OperatingSystem,
    /// 检测到的 Shell 类型
    pub shell: ShellType,
    /// 检测置信度 (0.0-1.0)
    pub confidence: f32,
    /// Shell 可执行文件路径（如果可用）
    pub shell_path: Option<String>,
    /// 检测来源描述
    pub detection_source: String,
}

impl ShellType {
    /// 获取 Shell 的友好名称
    pub fn friendly_name(&self) -> &'static str {
        match self {
            ShellType::Cmd => "Windows Command Prompt (cmd.exe)",
            ShellType::PowerShell => "Windows PowerShell",
            ShellType::GitBash => "Git Bash (MinGW/MSYS2)",
            ShellType::Wsl => "Windows Subsystem for Linux (WSL)",
            ShellType::Bash => "Bourne Again Shell (bash)",
            ShellType::Zsh => "Z Shell (zsh)",
            ShellType::Fish => "Friendly Interactive Shell (fish)",
            ShellType::Sh => "Bourne Shell (sh)",
            ShellType::Unknown => "Unknown Shell",
        }
    }

    /// 判断是否为 Windows Shell
    pub fn is_windows_shell(&self) -> bool {
        matches!(
            self,
            ShellType::Cmd | ShellType::PowerShell | ShellType::GitBash | ShellType::Wsl
        )
    }

    /// 判断是否为 Unix Shell
    pub fn is_unix_shell(&self) -> bool {
        matches!(
            self,
            ShellType::Bash | ShellType::Zsh | ShellType::Fish | ShellType::Sh
        )
    }

    /// 获取推荐的文件列表命令
    /// 注意：PowerShell 同时支持 "ls" 和 "dir"，这里返回最常用的
    pub fn list_files_command(&self) -> &'static str {
        match self {
            ShellType::Cmd => "dir",
            ShellType::PowerShell => "dir", // PowerShell 中 "dir" 更常用
            ShellType::GitBash => "ls",
            ShellType::Wsl => "ls",
            ShellType::Bash => "ls",
            ShellType::Zsh => "ls",
            ShellType::Fish => "ls",
            ShellType::Sh => "ls",
            ShellType::Unknown => "ls",
        }
    }

    /// 获取推荐的删除文件命令
    /// 注意：PowerShell 同时支持 "rm" 和 "del"，这里返回最常用的
    pub fn delete_file_command(&self) -> &'static str {
        match self {
            ShellType::Cmd => "del",
            ShellType::PowerShell => "del", // PowerShell 中 "del" 更常用
            ShellType::GitBash => "rm",
            ShellType::Wsl => "rm",
            ShellType::Bash => "rm",
            ShellType::Zsh => "rm",
            ShellType::Fish => "rm",
            ShellType::Sh => "rm",
            ShellType::Unknown => "rm",
        }
    }

    /// 获取推荐的复制文件命令
    /// 注意：PowerShell 同时支持 "cp" 和 "copy"，这里返回最常用的
    pub fn copy_file_command(&self) -> &'static str {
        match self {
            ShellType::Cmd => "copy",
            ShellType::PowerShell => "copy", // PowerShell 中 "copy" 更常用
            ShellType::GitBash => "cp",
            ShellType::Wsl => "cp",
            ShellType::Bash => "cp",
            ShellType::Zsh => "cp",
            ShellType::Fish => "cp",
            ShellType::Sh => "cp",
            ShellType::Unknown => "cp",
        }
    }

    /// 获取所有可能的文件列表命令（用于 AI 模型参考）
    pub fn list_files_commands(&self) -> &'static [&'static str] {
        match self {
            ShellType::Cmd => &["dir"],
            ShellType::PowerShell => &["dir", "ls"], // PowerShell 支持两者
            ShellType::GitBash => &["ls"],
            ShellType::Wsl => &["ls"],
            ShellType::Bash => &["ls"],
            ShellType::Zsh => &["ls"],
            ShellType::Fish => &["ls"],
            ShellType::Sh => &["ls"],
            ShellType::Unknown => &["ls"],
        }
    }

    /// 获取所有可能的删除文件命令（用于 AI 模型参考）
    pub fn delete_file_commands(&self) -> &'static [&'static str] {
        match self {
            ShellType::Cmd => &["del"],
            ShellType::PowerShell => &["del", "rm", "Remove-Item"], // PowerShell 支持多个
            ShellType::GitBash => &["rm"],
            ShellType::Wsl => &["rm"],
            ShellType::Bash => &["rm"],
            ShellType::Zsh => &["rm"],
            ShellType::Fish => &["rm"],
            ShellType::Sh => &["rm"],
            ShellType::Unknown => &["rm"],
        }
    }

    /// 获取所有可能的复制文件命令（用于 AI 模型参考）
    pub fn copy_file_commands(&self) -> &'static [&'static str] {
        match self {
            ShellType::Cmd => &["copy"],
            ShellType::PowerShell => &["copy", "cp", "Copy-Item"], // PowerShell 支持多个
            ShellType::GitBash => &["cp"],
            ShellType::Wsl => &["cp"],
            ShellType::Bash => &["cp"],
            ShellType::Zsh => &["cp"],
            ShellType::Fish => &["cp"],
            ShellType::Sh => &["cp"],
            ShellType::Unknown => &["cp"],
        }
    }

    /// 获取路径分隔符
    pub fn path_separator(&self) -> &'static str {
        match self {
            ShellType::Cmd => "\\",
            ShellType::PowerShell => "\\",
            ShellType::GitBash => "/",
            ShellType::Wsl => "/",
            ShellType::Bash => "/",
            ShellType::Zsh => "/",
            ShellType::Fish => "/",
            ShellType::Sh => "/",
            ShellType::Unknown => "/",
        }
    }
}

impl OperatingSystem {
    /// 检测当前操作系统
    pub fn detect() -> Self {
        match std::env::consts::OS {
            "windows" => OperatingSystem::Windows,
            "linux" => OperatingSystem::Linux,
            "macos" => OperatingSystem::MacOS,
            _ => OperatingSystem::Other,
        }
    }

    /// 获取友好名称
    pub fn friendly_name(&self) -> &'static str {
        match self {
            OperatingSystem::Windows => "Windows",
            OperatingSystem::Linux => "Linux",
            OperatingSystem::MacOS => "macOS",
            OperatingSystem::Other => "Unknown OS",
        }
    }
}

/// Shell 检测器
pub struct ShellDetector;

impl ShellDetector {
    /// 检测当前 Shell
    pub fn detect() -> SystemInfo {
        let os = OperatingSystem::detect();

        // 按优先级检测
        let detectors = [Self::detect_from_env, Self::detect_from_shell_features];

        for detector in detectors {
            if let Some(info) = detector(&os) {
                return info;
            }
        }

        // 如果都检测失败，返回默认值
        SystemInfo {
            os,
            shell: ShellType::Unknown,
            confidence: 0.0,
            shell_path: None,
            detection_source: "default".to_string(),
        }
    }

    /// 从环境变量检测
    fn detect_from_env(os: &OperatingSystem) -> Option<SystemInfo> {
        match os {
            OperatingSystem::Windows => {
                // Windows 检测逻辑
                if env::var("PROMPT").is_ok() && env::var("WINDIR").is_ok() {
                    return Some(SystemInfo {
                        os: os.clone(),
                        shell: ShellType::Cmd,
                        confidence: 0.8,
                        shell_path: env::var("COMSPEC").ok(),
                        detection_source: "env (PROMPT+WINDIR)".to_string(),
                    });
                }

                if env::var("PSModulePath").is_ok() {
                    return Some(SystemInfo {
                        os: os.clone(),
                        shell: ShellType::PowerShell,
                        confidence: 0.9,
                        shell_path: None,
                        detection_source: "env (PSModulePath)".to_string(),
                    });
                }

                if env::var("MSYSTEM").is_ok() || env::var("MINGW_PREFIX").is_ok() {
                    return Some(SystemInfo {
                        os: os.clone(),
                        shell: ShellType::GitBash,
                        confidence: 0.9,
                        shell_path: env::var("SHELL").ok(),
                        detection_source: "env (MSYSTEM/MINGW_PREFIX)".to_string(),
                    });
                }

                if env::var("WSL_DISTRO_NAME").is_ok() || env::var("WSL_INTEROP").is_ok() {
                    return Some(SystemInfo {
                        os: os.clone(),
                        shell: ShellType::Wsl,
                        confidence: 0.95,
                        shell_path: None,
                        detection_source: "env (WSL_DISTRO_NAME/WSL_INTEROP)".to_string(),
                    });
                }

                None
            }
            OperatingSystem::Linux | OperatingSystem::MacOS => {
                // Unix 检测逻辑
                if let Ok(shell) = env::var("SHELL") {
                    let shell_type = if shell.contains("zsh") {
                        ShellType::Zsh
                    } else if shell.contains("fish") {
                        ShellType::Fish
                    } else if shell.contains("bash") {
                        ShellType::Bash
                    } else {
                        ShellType::Sh
                    };

                    return Some(SystemInfo {
                        os: os.clone(),
                        shell: shell_type,
                        confidence: 0.85,
                        shell_path: Some(shell),
                        detection_source: "env (SHELL)".to_string(),
                    });
                }

                None
            }
            OperatingSystem::Other => None,
        }
    }

    /// 从 Shell 特定特征检测
    fn detect_from_shell_features(os: &OperatingSystem) -> Option<SystemInfo> {
        // 备用检测方法
        match os {
            OperatingSystem::Windows => {
                // Windows 备用检测
                Some(SystemInfo {
                    os: os.clone(),
                    shell: ShellType::PowerShell, // 现代 Windows 默认
                    confidence: 0.5,
                    shell_path: None,
                    detection_source: "fallback (Windows default)".to_string(),
                })
            }
            OperatingSystem::Linux | OperatingSystem::MacOS => {
                // Unix 备用检测
                Some(SystemInfo {
                    os: os.clone(),
                    shell: ShellType::Bash, // 最常见的 Unix Shell
                    confidence: 0.5,
                    shell_path: None,
                    detection_source: "fallback (Unix default)".to_string(),
                })
            }
            OperatingSystem::Other => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_type_friendly_name() {
        assert_eq!(
            ShellType::Cmd.friendly_name(),
            "Windows Command Prompt (cmd.exe)"
        );
        assert_eq!(ShellType::PowerShell.friendly_name(), "Windows PowerShell");
        assert_eq!(ShellType::Bash.friendly_name(), "Bourne Again Shell (bash)");
    }

    #[test]
    fn test_shell_type_is_windows() {
        assert!(ShellType::Cmd.is_windows_shell());
        assert!(ShellType::PowerShell.is_windows_shell());
        assert!(ShellType::GitBash.is_windows_shell());
        assert!(!ShellType::Bash.is_windows_shell());
    }

    #[test]
    fn test_shell_type_is_unix() {
        assert!(ShellType::Bash.is_unix_shell());
        assert!(ShellType::Zsh.is_unix_shell());
        assert!(!ShellType::Cmd.is_unix_shell());
    }

    #[test]
    fn test_shell_type_commands() {
        assert_eq!(ShellType::Cmd.list_files_command(), "dir");
        assert_eq!(ShellType::PowerShell.list_files_command(), "dir");
        assert_eq!(ShellType::Bash.list_files_command(), "ls");
        assert_eq!(ShellType::Cmd.delete_file_command(), "del");
        assert_eq!(ShellType::PowerShell.delete_file_command(), "del");
        assert_eq!(ShellType::Bash.delete_file_command(), "rm");
        assert_eq!(ShellType::Cmd.copy_file_command(), "copy");
        assert_eq!(ShellType::PowerShell.copy_file_command(), "copy");
        assert_eq!(ShellType::Bash.copy_file_command(), "cp");
    }

    #[test]
    fn test_shell_type_all_list_commands() {
        // 测试所有 Shell 类型的 list_files_command
        assert_eq!(ShellType::Cmd.list_files_command(), "dir");
        assert_eq!(ShellType::PowerShell.list_files_command(), "dir");
        assert_eq!(ShellType::GitBash.list_files_command(), "ls");
        assert_eq!(ShellType::Wsl.list_files_command(), "ls");
        assert_eq!(ShellType::Bash.list_files_command(), "ls");
        assert_eq!(ShellType::Zsh.list_files_command(), "ls");
        assert_eq!(ShellType::Fish.list_files_command(), "ls");
        assert_eq!(ShellType::Sh.list_files_command(), "ls");
        assert_eq!(ShellType::Unknown.list_files_command(), "ls");
    }

    #[test]
    fn test_shell_type_all_delete_commands() {
        // 测试所有 Shell 类型的 delete_file_command
        assert_eq!(ShellType::Cmd.delete_file_command(), "del");
        assert_eq!(ShellType::PowerShell.delete_file_command(), "del");
        assert_eq!(ShellType::GitBash.delete_file_command(), "rm");
        assert_eq!(ShellType::Wsl.delete_file_command(), "rm");
        assert_eq!(ShellType::Bash.delete_file_command(), "rm");
        assert_eq!(ShellType::Zsh.delete_file_command(), "rm");
        assert_eq!(ShellType::Fish.delete_file_command(), "rm");
        assert_eq!(ShellType::Sh.delete_file_command(), "rm");
        assert_eq!(ShellType::Unknown.delete_file_command(), "rm");
    }

    #[test]
    fn test_shell_type_all_copy_commands() {
        // 测试所有 Shell 类型的 copy_file_command
        assert_eq!(ShellType::Cmd.copy_file_command(), "copy");
        assert_eq!(ShellType::PowerShell.copy_file_command(), "copy");
        assert_eq!(ShellType::GitBash.copy_file_command(), "cp");
        assert_eq!(ShellType::Wsl.copy_file_command(), "cp");
        assert_eq!(ShellType::Bash.copy_file_command(), "cp");
        assert_eq!(ShellType::Zsh.copy_file_command(), "cp");
        assert_eq!(ShellType::Fish.copy_file_command(), "cp");
        assert_eq!(ShellType::Sh.copy_file_command(), "cp");
        assert_eq!(ShellType::Unknown.copy_file_command(), "cp");
    }

    #[test]
    fn test_shell_type_all_path_separators() {
        // 测试所有 Shell 类型的 path_separator
        assert_eq!(ShellType::Cmd.path_separator(), "\\");
        assert_eq!(ShellType::PowerShell.path_separator(), "\\");
        assert_eq!(ShellType::GitBash.path_separator(), "/");
        assert_eq!(ShellType::Wsl.path_separator(), "/");
        assert_eq!(ShellType::Bash.path_separator(), "/");
        assert_eq!(ShellType::Zsh.path_separator(), "/");
        assert_eq!(ShellType::Fish.path_separator(), "/");
        assert_eq!(ShellType::Sh.path_separator(), "/");
        assert_eq!(ShellType::Unknown.path_separator(), "/");
    }

    #[test]
    fn test_shell_type_powershell_multiple_commands() {
        // 测试 PowerShell 的多个命令选项
        assert_eq!(ShellType::PowerShell.list_files_commands(), &["dir", "ls"]);
        assert_eq!(
            ShellType::PowerShell.delete_file_commands(),
            &["del", "rm", "Remove-Item"]
        );
        assert_eq!(
            ShellType::PowerShell.copy_file_commands(),
            &["copy", "cp", "Copy-Item"]
        );
    }

    #[test]
    fn test_shell_type_debug_implementation() {
        // 测试 Debug trait 实现
        let shell_types = vec![
            ShellType::Cmd,
            ShellType::PowerShell,
            ShellType::GitBash,
            ShellType::Wsl,
            ShellType::Bash,
            ShellType::Zsh,
            ShellType::Fish,
            ShellType::Sh,
            ShellType::Unknown,
        ];

        for shell in shell_types {
            // 确保 Debug 可以正常工作
            let debug_str = format!("{:?}", shell);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_system_info_debug_implementation() {
        // 测试 SystemInfo 的 Debug trait
        let system_info = SystemInfo {
            os: OperatingSystem::Windows,
            shell: ShellType::PowerShell,
            confidence: 0.9,
            shell_path: Some(
                "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe".to_string(),
            ),
            detection_source: "test".to_string(),
        };

        let debug_str = format!("{:?}", system_info);
        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("Windows"));
        assert!(debug_str.contains("PowerShell"));
        assert!(debug_str.contains("0.9"));
    }

    #[test]
    fn test_operating_system_detect() {
        // 只测试我们可以预期的
        let os = OperatingSystem::detect();
        // 在 CI 中可能是任何 OS，所以我们只测试它不是 panic
        assert!(matches!(
            os,
            OperatingSystem::Windows
                | OperatingSystem::Linux
                | OperatingSystem::MacOS
                | OperatingSystem::Other
        ));
    }

    #[test]
    fn test_shell_detector_fallback_logic() {
        // 测试 fallback 逻辑（通过直接调用）
        let os = OperatingSystem::Windows;
        let fallback = ShellDetector::detect_from_shell_features(&os);

        assert!(fallback.is_some());
        let info = fallback.unwrap();
        assert_eq!(info.shell, ShellType::PowerShell);
        assert_eq!(info.confidence, 0.5);
        assert_eq!(info.detection_source, "fallback (Windows default)");
    }
}

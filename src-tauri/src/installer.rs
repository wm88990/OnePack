use crate::config::Package;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::process::Command;


// ===== 安装状态 =====

pub static INSTALL_ABORT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// ===== 安装结果 =====

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallResult {
    pub id: String,
    pub name: String,
    pub success: bool,
    pub message: String,
}

// ===== 安装执行引擎 =====

/// 执行单个软件的静默安装
pub fn install_package(app_dir: &Path, pkg: &Package) -> Result<(), String> {
    // 检查是否已请求停止
    if INSTALL_ABORT.load(std::sync::atomic::Ordering::Relaxed) {
        return Err("安装已取消".to_string());
    }

    let packages_dir = app_dir.join("packages");

    let installer_path = match pkg.source.source_type.as_str() {
        "local" => {
            match &pkg.source.path {
                Some(path) => packages_dir.join(path),
                None => return Err(format!("{}: 未指定安装包路径", pkg.name)),
            }
        }
        _ => return Err(format!("{}: 暂不支持远程安装", pkg.name)),
    };

    if !installer_path.exists() {
        return Err(format!("{}: 安装包不存在 ({})", pkg.name, installer_path.display()));
    }

    // 校验文件哈希（如果配置了）
    if let Some(ref expected_hash) = pkg.source.hash_sha256 {
        if !expected_hash.is_empty() {
            match verify_hash(&installer_path, expected_hash) {
                Ok(true) => {}
                Ok(false) => return Err(format!("{}: 文件校验失败", pkg.name)),
                Err(e) => return Err(format!("{}: 校验错误: {}", pkg.name, e)),
            }
        }
    }

    // 构建安装命令
    let cmd = match pkg.install.install_type.as_str() {
        "exe" => {
            installer_path.display().to_string()
        }
        "msi" => {
            format!("msiexec.exe /i {}", installer_path.display())
        }
        _ => {
            return Err(format!("{}: 不支持的安装类型: {}", pkg.name, pkg.install.install_type));
        }
    };

    let full_cmd = format!("{} {}", cmd, pkg.install.silent_args);

    // 执行安装（等待完成）
    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(["/C", &full_cmd])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("{}: 执行安装命令失败: {}", pkg.name, e))?;

    #[cfg(not(target_os = "windows"))]
    let result = Command::new("cmd")
        .args(["/C", &full_cmd])
        .output()
        .map_err(|e| format!("{}: 执行安装命令失败: {}", pkg.name, e))?;

    if result.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("{}: 安装程序返回错误: {}", pkg.name, stderr.trim()))
    }
}

/// 校验文件 SHA256 哈希
fn verify_hash(file_path: &Path, expected: &str) -> Result<bool, String> {
    let data = fs::read(file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let hash = format!("{:x}", hasher.finalize());

    Ok(hash.eq_ignore_ascii_case(expected))
}

/// 停止安装
pub fn stop_install() {
    INSTALL_ABORT.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// 重置停止标志
pub fn reset_abort() {
    INSTALL_ABORT.store(false, std::sync::atomic::Ordering::Relaxed);
}

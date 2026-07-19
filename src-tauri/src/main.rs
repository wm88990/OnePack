// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod installer;

use config::Config;
use installer::{install_package, reset_abort, stop_install, InstallResult};

use tauri::Emitter;

// ===== IPC 命令 =====

#[tauri::command]
fn load_packages(_app_handle: tauri::AppHandle) -> Result<Config, String> {
    let app_dir = config::get_app_dir();
    config::load_config(&app_dir)
}

#[tauri::command]
async fn start_install(app_handle: tauri::AppHandle, package_ids: Vec<String>) -> Result<String, String> {
    reset_abort();

    let app_dir = config::get_app_dir();
    let config = config::load_config(&app_dir)?;

    // 过滤出选中的且已启用的包，保持配置文件中的顺序
    let selected: Vec<_> = config.packages.iter()
        .filter(|p| package_ids.contains(&p.id) && p.enabled)
        .collect();

    let total = selected.len() as u32;
    let mut results: Vec<InstallResult> = Vec::new();

    // 发送开始事件
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "type": "start",
        "total": total,
        "message": format!("准备安装 {} 款软件", total),
    }));

    for (i, pkg) in selected.iter().enumerate() {
        if installer::INSTALL_ABORT.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = app_handle.emit("install-progress", serde_json::json!({
                "type": "error",
                "package_id": pkg.id,
                "package_name": pkg.name,
                "message": "安装已取消",
            }));
            results.push(InstallResult {
                id: pkg.id.clone(),
                name: pkg.name.clone(),
                success: false,
                message: "安装已取消".to_string(),
            });
            continue;
        }

        // 发送进度事件
        let _ = app_handle.emit("install-progress", serde_json::json!({
            "type": "progress",
            "package_id": pkg.id,
            "package_name": pkg.name.clone(),
            "current": i + 1,
            "total": total,
            "message": format!("正在安装: {}", pkg.name),
        }));

        // 执行安装
        match install_package(&app_dir, pkg) {
            Ok(_) => {
                let _ = app_handle.emit("install-progress", serde_json::json!({
                    "type": "success",
                    "package_id": pkg.id.clone(),
                    "package_name": pkg.name.clone(),
                    "current": i + 1,
                    "total": total,
                }));
                results.push(InstallResult {
                    id: pkg.id.clone(),
                    name: pkg.name.clone(),
                    success: true,
                    message: "安装成功".to_string(),
                });
            }
            Err(e) => {
                let _ = app_handle.emit("install-progress", serde_json::json!({
                    "type": "error",
                    "package_id": pkg.id.clone(),
                    "package_name": pkg.name.clone(),
                    "message": e,
                }));
                results.push(InstallResult {
                    id: pkg.id.clone(),
                    name: pkg.name.clone(),
                    success: false,
                    message: e,
                });
            }
        }
    }

    // 发送完成事件
    let success_count = results.iter().filter(|r| r.success).count();
    let failed_count = results.iter().filter(|r| !r.success).count();
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "type": "complete",
        "success": success_count,
        "failed": failed_count,
        "total": total,
        "message": format!("安装完成：成功 {}，失败 {}", success_count, failed_count),
    }));

    Ok(format!("安装完成：成功 {}，失败 {}", success_count, failed_count))
}

#[tauri::command]
fn stop_install_cmd() -> Result<String, String> {
    stop_install();
    Ok("已发送停止信号".to_string())
}

#[tauri::command]
/// 扫描 packages/ 目录中的文件，自动识别并生成配置条目
#[tauri::command]
fn scan_packages(_app_handle: tauri::AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let app_dir = config::get_app_dir();
    let packages_dir = app_dir.join("packages");

    if !packages_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&packages_dir)
        .map_err(|e| format!("读取 packages 目录失败: {}", e))?;

    let mut results: Vec<serde_json::Value> = Vec::new();

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let extension = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        // 跳过非安装包文件
        if !matches!(extension.as_str(), "exe" | "msi" | "zip") {
            continue;
        }

        // 生成包 ID（去掉扩展名，转为小写，替换空格和特殊字符）
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let pkg_id = stem.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>();

        // 推断安装类型和默认参数
        let (install_type, silent_args, main_exe, extract_subdir) = match extension.as_str() {
            "zip" => (
                "green",
                "",
                Some(stem.to_string()),
                Some(stem.to_string()),
            ),
            "msi" => (
                "msi",
                String::new(),
                None,
                None,
            ),
            "exe" => {
                // 根据文件名推断静默参数
                let name_lower = file_name.to_lowercase();
                let silent = if name_lower.contains("chrome") {
                    "/silent /install".to_string()
                } else if name_lower.contains("vscode") || name_lower.contains("code") {
                    "/VERYSILENT /MERGETASKS=!RUN /NORESTART".to_string()
                } else if name_lower.contains("git") {
                    "/VERYSILENT /NORESTART".to_string()
                } else {
                    "/S".to_string() // 默认 NSIS 静默参数
                };
                ("exe", silent, None, None)
            }
            _ => continue,
        };

        // 获取文件大小
        let file_size = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(0);
        let size_str = if file_size > 1_000_000 {
            format!("{:.1} MB", file_size as f64 / 1_000_000.0)
        } else {
            format!("{} KB", file_size / 1024)
        };

        results.push(serde_json::json!({
            "id": pkg_id,
            "name": stem.to_string(),
            "icon": "📦",
            "category": "system",
            "description": format!("从文件自动检测: {}", file_name),
            "version": "自动检测",
            "size": size_str,
            "enabled": true,
            "source": {
                "type": "local",
                "path": file_name,
                "hash_sha256": ""
            },
            "install": {
                "type": install_type,
                "silent_args": silent_args,
                "main_exe": main_exe,
                "extract_subdir": extract_subdir,
                "create_shortcut": false
            }
        }));
    }

    Ok(results)
}

/// 将扫描到的软件包追加到 config.yaml
#[tauri::command]
fn add_packages(_app_handle: tauri::AppHandle, packages: Vec<serde_json::Value>) -> Result<String, String> {
    let app_dir = config::get_app_dir();
    let config_path = app_dir.join("config.yaml");

    // 读取现有配置
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置失败: {}", e))?;
    let mut config: Config = serde_yaml::from_str(&content)
        .map_err(|e| format!("解析配置失败: {}", e))?;

    // 检查重复（根据 id），只添加新的
    let existing_ids: std::collections::HashSet<String> = config.packages.iter()
        .map(|p| p.id.clone())
        .collect();

    let mut added = 0;
    for pkg_val in &packages {
        let pkg: config::Package = serde_json::from_value(pkg_val.clone())
            .map_err(|e| format!("解析包数据失败: {}", e))?;

        if !existing_ids.contains(&pkg.id) {
            config.packages.push(pkg);
            added += 1;
        }
    }

    if added > 0 {
        // 保存配置
        let yaml = serde_yaml::to_string(&config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        std::fs::write(&config_path, yaml)
            .map_err(|e| format!("写入配置失败: {}", e))?;
    }

    Ok(format!("已添加 {} 个软件包", added))
}

#[tauri::command]
fn export_config(_app_handle: tauri::AppHandle, config_data: serde_json::Value) -> Result<String, String> {
    let app_dir = config::get_app_dir();
    let export_path = app_dir.join("config-export.yaml");

    // 将 JSON 转为 YAML
    let yaml_value: serde_yaml::Value = serde_json::from_value(config_data)
        .map_err(|e| format!("转换配置数据失败: {}", e))?;
    let yaml_str = serde_yaml::to_string(&yaml_value)
        .map_err(|e| format!("序列化 YAML 失败: {}", e))?;

    std::fs::write(&export_path, yaml_str)
        .map_err(|e| format!("写入导出文件失败: {}", e))?;

    Ok(format!("已导出到: {}", export_path.display()))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            load_packages,
            start_install,
            stop_install_cmd,
            export_config,
            scan_packages,
            add_packages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

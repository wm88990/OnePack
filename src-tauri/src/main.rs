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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

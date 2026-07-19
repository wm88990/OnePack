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
                String::new(),
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

        // 智能检测：尝试从文件名提取版本号
        let version = extract_version_from_filename(stem);

        // 智能检测：尝试从 exe 文件读取版本信息 (仅 Windows)
        let (detected_version, detected_vendor) = if extension == "exe" && cfg!(target_os = "windows") {
            read_exe_version_info(&path)
        } else {
            (String::new(), String::new())
        };

        let final_version = if !detected_version.is_empty() {
            detected_version
        } else if !version.is_empty() {
            version
        } else {
            "未知".to_string()
        };

        // 智能分类：根据文件名猜测分类
        let category = guess_category(&file_name.to_lowercase(), &extension);

        // 生成更友好的名称
        let friendly_name = make_friendly_name(stem);

        // 生成描述
        let description = if !detected_vendor.is_empty() {
            format!("{} - {}", detected_vendor, final_version)
        } else {
            format!("从文件自动检测: {}", file_name)
        };

        // 为 exe/msi 计算 SHA256
        let hash = if matches!(extension.as_str(), "exe" | "msi") {
            compute_file_hash(&path)
        } else {
            String::new()
        };

        results.push(serde_json::json!({
            "id": pkg_id,
            "name": friendly_name,
            "icon": guess_icon(&category),
            "category": category,
            "description": description,
            "version": final_version,
            "size": size_str,
            "enabled": true,
            "source": {
                "type": "local",
                "path": file_name,
                "hash_sha256": hash
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

/// 从文件名提取版本号
fn extract_version_from_filename(stem: &str) -> String {
    // 匹配常见版本号模式: v1.0, 1.0.0, 2024.1, x64 等
    let re = regex::Regex::new(r"(\d{1,4}[.-]\d{1,2}(?:[.-]\d{1,2})?(?:[.-]\d{1,2})?)").unwrap();
    if let Some(caps) = re.find(stem) {
        return caps.as_str().trim_start_matches('v').trim_start_matches('V').to_string();
    }
    String::new()
}

/// 根据文件名猜测软件分类
fn guess_category(filename_lower: &str, extension: &str) -> String {
    if extension == "zip" {
        // zip 文件默认归为绿色软件
        return "green".to_string();
    }
    let browser_keywords = ["chrome", "firefox", "edge", "opera", "brave", "vivaldi"];
    let office_keywords = ["office", "word", "excel", "ppt", "wps", "libreoffice"];
    let social_keywords = ["wechat", "qq", "telegram", "discord", "tim", "dingtalk", "wechat"];
    let media_keywords = ["vlc", "potplayer", "foobar", "aimp", "spotify", "music", "video", "mpc", "kodi"];
    let dev_keywords = ["vscode", "code", "git", "node", "python", "java", "rust", "go", "docker", "idea", "clion", "webstorm", "notepad++", "sublime"];
    let system_keywords = ["7-zip", "7z", "winrar", "bandizip", "huorong", "360", "kaspersky", "driver", "directx"];

    let keywords_list: [(&str, &[&str]); 6] = [
        ("browser", &browser_keywords),
        ("office", &office_keywords),
        ("social", &social_keywords),
        ("media", &media_keywords),
        ("dev", &dev_keywords),
        ("system", &system_keywords),
    ];

    for (cat, keywords) in keywords_list {
        if keywords.iter().any(|k| filename_lower.contains(k)) {
            return cat.to_string();
        }
    }

    "system".to_string()
}

/// 根据分类猜测图标
fn guess_icon(category: &str) -> String {
    match category {
        "browser" => "🌐".to_string(),
        "office" => "📄".to_string(),
        "social" => "💬".to_string(),
        "media" => "🎵".to_string(),
        "dev" => "🛠️".to_string(),
        "green" => "🍃".to_string(),
        _ => "📦".to_string(),
    }
}

/// 从文件名生成更友好的软件名
fn make_friendly_name(stem: &str) -> String {
    let name = stem.to_string();
    // 去掉常见后缀: Setup, Install, x64, x86, portable 等
    let suffixes = [
        "-x64", "_x64", "-x86", "_x86", "-64bit", "-32bit",
        "Setup", "Install", "Portable", ".portable",
    ];
    let mut result = name.clone();
    for suffix in &suffixes {
        if result.to_lowercase().ends_with(&suffix.to_lowercase()) {
            result = result[..result.len() - suffix.len()].to_string();
        }
    }
    // 去掉末尾的 _- . 等分隔符
    result.trim_end_matches('_').trim_end_matches('-').trim_end_matches('.').to_string()
}

/// 计算 SHA256
fn compute_file_hash(path: &std::path::Path) -> String {
    match std::fs::read(path) {
        Ok(data) => {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&data);
            format!("{:x}", hasher.finalize())
        }
        Err(_) => String::new(),
    }
}

/// 从 exe 文件读取版本信息 (仅 Windows)
#[cfg(target_os = "windows")]
fn read_exe_version_info(path: &std::path::Path) -> (String, String) {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    let path_str = path.display().to_string();

    // 使用 PowerShell 读取文件版本信息
    let ps_cmd = format!(
        "$v = [System.Diagnostics.FileVersionInfo]::GetVersionInfo('{}'); Write-Output \"$($v.FileVersion)|$($v.CompanyName)\"",
        path_str.replace("'", "''")
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .creation_flags(0x08000000)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let parts: Vec<&str> = stdout.splitn(2, '|').collect();
            let version = parts.get(0).map(|s| s.trim().to_string()).unwrap_or_default();
            let vendor = parts.get(1).map(|s| s.trim().to_string()).unwrap_or_default();
            (version, vendor)
        }
        _ => (String::new(), String::new()),
    }
}

#[cfg(not(target_os = "windows"))]
fn read_exe_version_info(_path: &std::path::Path) -> (String, String) {
    (String::new(), String::new())
}

/// 检查指定软件是否已安装
#[tauri::command]
fn check_installed(_app_handle: tauri::AppHandle, packages: Vec<String>) -> Result<std::collections::HashMap<String, bool>, String> {
    let mut result = std::collections::HashMap::new();

    let app_dir = config::get_app_dir();
    let config = config::load_config(&app_dir)?;

    for pkg_id in &packages {
        let installed = is_package_installed(&app_dir, &config, pkg_id);
        result.insert(pkg_id.clone(), installed);
    }

    Ok(result)
}

/// 检查单个软件是否已安装
#[cfg(target_os = "windows")]
fn is_package_installed(app_dir: &std::path::Path, config: &Config, pkg_id: &str) -> bool {
    let pkg = match config.packages.iter().find(|p| p.id == pkg_id) {
        Some(p) => p,
        None => return false,
    };

    let packages_dir = app_dir.join("packages");
    let apps_dir = app_dir.join("apps");

    match pkg.install.install_type.as_str() {
        // 绿色软件：检查 apps/ 目录下是否存在对应子目录
        "green" => {
            let subdir = pkg.install.extract_subdir.as_deref().unwrap_or(&pkg.id);
            let target = apps_dir.join(subdir);
            if let Some(ref main_exe) = pkg.install.main_exe {
                if !main_exe.is_empty() {
                    return target.join(main_exe).exists();
                }
            }
            target.exists()
        }
        // exe/msi 安装的软件：检查安装目录或注册表
        "exe" | "msi" => {
            // 方法1：检查配置中指定的安装目录
            if let Some(ref install_dir) = pkg.install.install_dir {
                if !install_dir.is_empty() {
                    let dir = std::path::Path::new(install_dir);
                    if dir.exists() {
                        // 目录存在，尝试找匹配的 exe
                        if dir.join(format!("{}.exe", pkg_id)).exists() {
                            return true;
                        }
                        // 检查目录是否有内容（简单启发式）
                        if let Ok(entries) = std::fs::read_dir(dir) {
                            if entries.count() > 0 {
                                return true;
                            }
                        }
                    }
                }
            }

            // 方法2：检查安装包是否仍在 packages/ 目录（未安装）
            if let Some(ref path) = pkg.source.path {
                if packages_dir.join(path).exists() {
                    return false; // 安装包还在，可能尚未安装
                }
            }

            // 方法3：尝试通过卸载注册表查找
            check_registry_for_app(&pkg.name, &pkg.id)
        }
        _ => false,
    }
}

#[cfg(not(target_os = "windows"))]
fn is_package_installed(_app_dir: &std::path::Path, _config: &Config, _pkg_id: &str) -> bool {
    false // 非 Windows 暂不支持
}

/// 通过 Windows 注册表检查软件是否已安装
#[cfg(target_os = "windows")]
fn check_registry_for_app(name: &str, id: &str) -> bool {
    use std::os::windows::process::CommandExt;

    // 查询注册表卸载项
    let reg_paths = [
        r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    for reg_path in &reg_paths {
        let output = std::process::Command::new("reg")
            .args(["query", reg_path, "/s", "/f", "DisplayName"])
            .creation_flags(0x08000000)
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // 检查是否包含软件名（不区分大小写）
                if stdout.to_lowercase().contains(&name.to_lowercase()) {
                    return true;
                }
                // 也检查包 ID
                if stdout.to_lowercase().contains(&id.to_lowercase()) {
                    return true;
                }
            }
        }
    }

    false
}

/// 更新单个软件包的配置（保存到 config.yaml）
#[tauri::command]
fn update_package(_app_handle: tauri::AppHandle, package_data: serde_json::Value) -> Result<String, String> {
    let app_dir = config::get_app_dir();
    let config_path = app_dir.join("config.yaml");

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置失败: {}", e))?;
    let mut config: Config = serde_yaml::from_str(&content)
        .map_err(|e| format!("解析配置失败: {}", e))?;

    let pkg: config::Package = serde_json::from_value(package_data.clone())
        .map_err(|e| format!("解析包数据失败: {}", e))?;

    if let Some(existing) = config.packages.iter_mut().find(|p| p.id == pkg.id) {
        *existing = pkg;
    } else {
        return Err(format!("未找到软件包: {}", package_data["id"]));
    }

    let yaml = serde_yaml::to_string(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, yaml)
        .map_err(|e| format!("写入配置失败: {}", e))?;

    Ok(format!("已更新: {}", config_path.display()))
}

/// 删除软件包
#[tauri::command]
fn delete_packages(_app_handle: tauri::AppHandle, package_ids: Vec<String>) -> Result<String, String> {
    let app_dir = config::get_app_dir();
    let config_path = app_dir.join("config.yaml");

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置失败: {}", e))?;
    let mut config: Config = serde_yaml::from_str(&content)
        .map_err(|e| format!("解析配置失败: {}", e))?;

    let ids_set: std::collections::HashSet<String> = package_ids.into_iter().collect();
    let before = config.packages.len();
    config.packages.retain(|p| !ids_set.contains(&p.id));
    let removed = before - config.packages.len();

    let yaml = serde_yaml::to_string(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, yaml)
        .map_err(|e| format!("写入配置失败: {}", e))?;

    Ok(format!("已删除 {} 个软件包", removed))
}

/// 批量更新软件包的安装目录
#[tauri::command]
fn batch_update_install_dir(_app_handle: tauri::AppHandle, package_ids: Vec<String>, install_dir: String, dir_format: String) -> Result<String, String> {
    let app_dir = config::get_app_dir();
    let config_path = app_dir.join("config.yaml");

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置失败: {}", e))?;
    let mut config: Config = serde_yaml::from_str(&content)
        .map_err(|e| format!("解析配置失败: {}", e))?;

    let ids_set: std::collections::HashSet<String> = package_ids.into_iter().collect();
    let mut updated = 0;
    for pkg in &mut config.packages {
        if ids_set.contains(&pkg.id) {
            pkg.install.install_dir = Some(install_dir.clone());
            pkg.install.dir_format = Some(dir_format.clone());
            updated += 1;
        }
    }

    let yaml = serde_yaml::to_string(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, yaml)
        .map_err(|e| format!("写入配置失败: {}", e))?;

    Ok(format!("已更新 {} 个软件包的安装目录", updated))
}

/// 批量更新软件包的分类
#[tauri::command]
fn batch_update_category(_app_handle: tauri::AppHandle, package_ids: Vec<String>, category: String) -> Result<String, String> {
    let app_dir = config::get_app_dir();
    let config_path = app_dir.join("config.yaml");

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置失败: {}", e))?;
    let mut config: Config = serde_yaml::from_str(&content)
        .map_err(|e| format!("解析配置失败: {}", e))?;

    let ids_set: std::collections::HashSet<String> = package_ids.into_iter().collect();
    let mut updated = 0;
    for pkg in &mut config.packages {
        if ids_set.contains(&pkg.id) {
            pkg.category = category.clone();
            updated += 1;
        }
    }

    let yaml = serde_yaml::to_string(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, yaml)
        .map_err(|e| format!("写入配置失败: {}", e))?;

    Ok(format!("已更新 {} 个软件包的分类", updated))
}

/// 导入配置文件（合并到当前配置）
#[tauri::command]
fn import_config(_app_handle: tauri::AppHandle, import_path: String) -> Result<Config, String> {
    let app_dir = config::get_app_dir();
    let import = std::path::Path::new(&import_path);

    if !import.exists() {
        return Err(format!("导入文件不存在: {}", import_path));
    }

    let import_content = std::fs::read_to_string(import)
        .map_err(|e| format!("读取导入文件失败: {}", e))?;
    let import_config: Config = serde_yaml::from_str(&import_content)
        .map_err(|e| format!("解析导入文件失败: {}", e))?;

    // 合并：新配置覆盖当前配置（保留当前配置的结构，用导入数据更新匹配项）
    let config_path = app_dir.join("config.yaml");
    let current_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取当前配置失败: {}", e))?;
    let mut current: Config = serde_yaml::from_str(&current_content)
        .map_err(|e| format!("解析当前配置失败: {}", e))?;

    // 更新 meta
    current.meta.name = import_config.meta.name;
    current.meta.version = import_config.meta.version;

    // 合并分类（添加新的分类）
    let existing_cat_ids: std::collections::HashSet<String> = current.categories.iter().map(|c| c.id.clone()).collect();
    for cat in import_config.categories {
        if !existing_cat_ids.contains(&cat.id) {
            current.categories.push(cat);
        }
    }

    // 合并软件包（按 id 更新或添加）
    let existing_pkg_ids: std::collections::HashSet<String> = current.packages.iter().map(|p| p.id.clone()).collect();
    for pkg in import_config.packages {
        if let Some(existing) = current.packages.iter_mut().find(|p| p.id == pkg.id) {
            *existing = pkg;
        } else {
            current.packages.push(pkg);
        }
    }

    // 保存合并后的配置
    let yaml = serde_yaml::to_string(&current)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, yaml)
        .map_err(|e| format!("写入配置失败: {}", e))?;

    Ok(current)
}

/// 对比两个配置，返回差异信息
#[tauri::command]
fn compare_configs(_app_handle: tauri::AppHandle, compare_path: String) -> Result<serde_json::Value, String> {
    let app_dir = config::get_app_dir();
    let config_path = app_dir.join("config.yaml");
    let compare = std::path::Path::new(&compare_path);

    if !compare.exists() {
        return Err(format!("对比文件不存在: {}", compare_path));
    }

    let current: Config = {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("读取当前配置失败: {}", e))?;
        serde_yaml::from_str(&content)
            .map_err(|e| format!("解析当前配置失败: {}", e))?
    };

    let other: Config = {
        let content = std::fs::read_to_string(compare)
            .map_err(|e| format!("读取对比配置失败: {}", e))?;
        serde_yaml::from_str(&content)
            .map_err(|e| format!("解析对比配置失败: {}", e))?
    };

    // 构建对比结果
    let current_ids: std::collections::HashSet<&str> = current.packages.iter().map(|p| p.id.as_str()).collect();
    let other_ids: std::collections::HashSet<&str> = other.packages.iter().map(|p| p.id.as_str()).collect();

    // 仅在当前配置中存在的包
    let only_current: Vec<serde_json::Value> = current.packages.iter()
        .filter(|p| !other_ids.contains(p.id.as_str()))
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect();

    // 仅在对比配置中存在的包
    let only_other: Vec<serde_json::Value> = other.packages.iter()
        .filter(|p| !current_ids.contains(p.id.as_str()))
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect();

    // 两个配置都存在但有差异的包
    let mut different: Vec<serde_json::Value> = Vec::new();
    for pkg in &current.packages {
        if let Some(other_pkg) = other.packages.iter().find(|p| p.id == pkg.id) {
            let cur_json = serde_json::to_value(pkg).unwrap_or_default();
            let oth_json = serde_json::to_value(other_pkg).unwrap_or_default();
            if cur_json != oth_json {
                different.push(serde_json::json!({
                    "id": pkg.id,
                    "name": pkg.name,
                    "current": cur_json,
                    "other": oth_json,
                    "changes": compare_package_diff(pkg, other_pkg)
                }));
            }
        }
    }

    Ok(serde_json::json!({
        "current_name": current.meta.name,
        "other_name": other.meta.name,
        "current_count": current.packages.len(),
        "other_count": other.packages.len(),
        "only_in_current": only_current,
        "only_in_other": only_other,
        "different": different,
    }))
}

/// 对比两个包的差异字段
fn compare_package_diff(a: &config::Package, b: &config::Package) -> Vec<serde_json::Value> {
    let mut changes = Vec::new();

    macro_rules! check_field {
        ($name:expr, $a:expr, $b:expr) => {
            if $a != $b {
                changes.push(serde_json::json!({
                    "field": $name,
                    "current": $a,
                    "other": $b,
                }));
            }
        };
    }

    check_field!("name", a.name, b.name);
    check_field!("version", a.version, b.version);
    check_field!("category", a.category, b.category);
    check_field!("description", a.description, b.description);
    check_field!("enabled", a.enabled, b.enabled);
    check_field!("install_type", a.install.install_type, b.install.install_type);
    check_field!("silent_args", a.install.silent_args, b.install.silent_args);
    check_field!("install_dir", a.install.install_dir, b.install.install_dir);
    check_field!("create_shortcut", a.install.create_shortcut, b.install.create_shortcut);

    changes
}

/// 读取当前配置文件的原始 YAML 内容
#[tauri::command]
fn get_config_yaml(_app_handle: tauri::AppHandle) -> Result<String, String> {
    let app_dir = config::get_app_dir();
    let config_path = app_dir.join("config.yaml");
    std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))
}

/// 保存原始 YAML 内容到配置文件
#[tauri::command]
fn save_config_yaml(_app_handle: tauri::AppHandle, yaml_content: String) -> Result<String, String> {
    let app_dir = config::get_app_dir();
    let config_path = app_dir.join("config.yaml");

    // 先验证 YAML 是否合法
    let _: Config = serde_yaml::from_str(&yaml_content)
        .map_err(|e| format!("YAML 格式错误: {}", e))?;

    std::fs::write(&config_path, yaml_content)
        .map_err(|e| format!("写入配置失败: {}", e))?;

    Ok("配置已保存".to_string())
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
            check_installed,
            update_package,
            delete_packages,
            batch_update_install_dir,
            batch_update_category,
            import_config,
            compare_configs,
            get_config_yaml,
            save_config_yaml,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

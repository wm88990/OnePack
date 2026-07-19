use crate::config::Package;
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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

/// 执行单个软件的安装（静默安装或绿色解压）
pub fn install_package(app_dir: &Path, pkg: &Package) -> Result<(), String> {
    if INSTALL_ABORT.load(std::sync::atomic::Ordering::Relaxed) {
        return Err("安装已取消".to_string());
    }

    let packages_dir = app_dir.join("packages");
    let apps_dir = app_dir.join("apps");

    match pkg.install.install_type.as_str() {
        "green" => install_green(&packages_dir, &apps_dir, pkg),
        "exe" => install_exe(&packages_dir, pkg),
        "msi" => install_msi(&packages_dir, pkg),
        _ => Err(format!("{}: 不支持的安装类型: {}", pkg.name, pkg.install.install_type)),
    }
}

/// 绿色软件解压：从 packages/ 解压 zip 到 apps/<子目录>/
fn install_green(packages_dir: &Path, apps_dir: &Path, pkg: &Package) -> Result<(), String> {
    let zip_path = match &pkg.source.path {
        Some(path) => packages_dir.join(path),
        None => return Err(format!("{}: 未指定安装包路径", pkg.name)),
    };

    if !zip_path.exists() {
        return Err(format!("{}: 安装包不存在 ({})", pkg.name, zip_path.display()));
    }

    // 校验文件哈希
    if let Some(ref expected_hash) = pkg.source.hash_sha256 {
        if !expected_hash.is_empty() {
            match verify_hash(&zip_path, expected_hash) {
                Ok(true) => {}
                Ok(false) => return Err(format!("{}: 文件校验失败", pkg.name)),
                Err(e) => return Err(format!("{}: 校验错误: {}", pkg.name, e)),
            }
        }
    }

    // 确定解压目标目录
    let subdir = pkg.install.extract_subdir.as_deref().unwrap_or(&pkg.id);
    let target_dir = apps_dir.join(subdir);

    // 如果目标目录已存在，先清空
    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)
            .map_err(|e| format!("{}: 清理旧目录失败: {}", pkg.name, e))?;
    }

    // 创建目标目录
    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("{}: 创建目录失败: {}", pkg.name, e))?;

    // 解压 zip 文件
    extract_zip(&zip_path, &target_dir)
        .map_err(|e| format!("{}: 解压失败: {}", pkg.name, e))?;

    // 如果配置了 main_exe，在桌面创建快捷方式（仅 Windows）
    #[cfg(target_os = "windows")]
    if let Some(ref main_exe) = pkg.install.main_exe {
        if !main_exe.is_empty() {
            let _ = create_shortcut(&pkg.name, &target_dir, main_exe);
        }
    }

    Ok(())
}

/// 静默安装 exe
fn install_exe(packages_dir: &Path, pkg: &Package) -> Result<(), String> {
    let installer_path = match &pkg.source.path {
        Some(path) => packages_dir.join(path),
        None => return Err(format!("{}: 未指定安装包路径", pkg.name)),
    };

    if !installer_path.exists() {
        return Err(format!("{}: 安装包不存在 ({})", pkg.name, installer_path.display()));
    }

    // 校验文件哈希
    if let Some(ref expected_hash) = pkg.source.hash_sha256 {
        if !expected_hash.is_empty() {
            match verify_hash(&installer_path, expected_hash) {
                Ok(true) => {}
                Ok(false) => return Err(format!("{}: 文件校验失败", pkg.name)),
                Err(e) => return Err(format!("{}: 校验错误: {}", pkg.name, e)),
            }
        }
    }

    // 构建安装命令（含自定义目录参数）
    let mut full_cmd = format!("\"{}\" {}", installer_path.display(), pkg.install.silent_args);
    if let Some(ref install_dir) = pkg.install.install_dir {
        if !install_dir.is_empty() {
            let dir_format = pkg.install.dir_format.as_deref().unwrap_or("/D=");
            full_cmd = format!("{} {}\"{}\"", full_cmd, dir_format, install_dir);
        }
    }

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

/// 静默安装 msi
fn install_msi(packages_dir: &Path, pkg: &Package) -> Result<(), String> {
    let installer_path = match &pkg.source.path {
        Some(path) => packages_dir.join(path),
        None => return Err(format!("{}: 未指定安装包路径", pkg.name)),
    };

    if !installer_path.exists() {
        return Err(format!("{}: 安装包不存在 ({})", pkg.name, installer_path.display()));
    }

    let full_cmd = format!(
        "msiexec.exe /i \"{}\" {} /qn",
        installer_path.display(),
        pkg.install.silent_args
    );

    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(["/C", &full_cmd])
        .creation_flags(0x08000000)
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

/// 解压 zip 文件到目标目录
fn extract_zip(zip_path: &Path, target_dir: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("打开 zip 失败: {}", e))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("读取 zip 失败: {}", e))?;

    for i in 0..archive.len() {
        if INSTALL_ABORT.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("解压已取消".to_string());
        }

        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?;

        let outpath = match entry.enclosed_name() {
            Some(path) => {
                // 跳过 macOS 资源 fork 等隐藏文件
                let name = path.to_string_lossy();
                if name.starts_with("__MACOSX") || name.contains("/._") {
                    continue;
                }
                target_dir.join(path)
            }
            None => continue,
        };

        if entry.is_dir() {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }

    Ok(())
}

/// 在桌面创建快捷方式（仅 Windows）
#[cfg(target_os = "windows")]
fn create_shortcut(name: &str, app_dir: &Path, main_exe: &str) -> Result<(), String> {
    use std::io::Write;

    let exe_path = app_dir.join(main_exe);
    if !exe_path.exists() {
        return Err(format!("主程序不存在: {}", exe_path.display()));
    }

    // 获取桌面路径
    let desktop = dirs_desktop();

    // 创建 .url 快捷方式文件
    let shortcut_path = desktop.join(format!("{}.url", name));
    let url_content = format!(
        "[InternetShortcut]\nURL=file:///{}/{}\nIconIndex=0\nIconFile={}\n",
        app_dir.display().to_string().replace('\\', "/"),
        main_exe,
        exe_path.display()
    );

    let mut f = fs::File::create(&shortcut_path)
        .map_err(|e| format!("创建快捷方式失败: {}", e))?;
    f.write_all(url_content.as_bytes())
        .map_err(|e| format!("写入快捷方式失败: {}", e))?;

    Ok(())
}

/// 获取 Windows 公共桌面路径
#[cfg(target_os = "windows")]
fn dirs_desktop() -> std::path::PathBuf {
    // C:\Users\Public\Desktop（所有用户）
    if let Ok(pub_desktop) = std::env::var("PUBLIC") {
        let desktop = std::path::PathBuf::from(pub_desktop).join("Desktop");
        if desktop.exists() {
            return desktop;
        }
    }
    // 回退到用户桌面
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        return std::path::PathBuf::from(userprofile).join("Desktop");
    }
    std::path::PathBuf::from(".")
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

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ===== 数据结构 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub meta: Meta,
    pub categories: Vec<Category>,
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub created: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon: String,
    pub category: String,
    pub description: String,
    pub version: String,
    pub size: String,
    pub source: PackageSource,
    pub install: PackageInstall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub hash_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInstall {
    #[serde(rename = "type")]
    pub install_type: String,
    pub silent_args: String,
    #[serde(default)]
    pub manual_args: Option<String>,
}

// ===== 配置加载 =====

/// 加载 config.yaml 配置文件并扫描 packages/ 目录
pub fn load_config(app_dir: &Path) -> Result<Config, String> {
    let config_path = app_dir.join("config.yaml");

    if !config_path.exists() {
        return Err(format!("配置文件不存在: {}", config_path.display()));
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let mut config: Config = serde_yaml::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 扫描 packages/ 目录，标记哪些包有本地安装包文件
    let packages_dir = app_dir.join("packages");
    if packages_dir.exists() {
        for pkg in &mut config.packages {
            if pkg.source.source_type == "local" {
                if let Some(ref path) = pkg.source.path {
                    let full_path = packages_dir.join(path);
                    pkg.source.path = Some(format!(
                        "{}",
                        full_path.exists()
                    ));
                }
            }
        }
    }

    Ok(config)
}

/// 导出当前配置为 YAML 文件
#[allow(dead_code)]
pub fn export_config(config: &Config, output_path: &Path) -> Result<(), String> {
    let yaml = serde_yaml::to_string(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    fs::write(output_path, yaml)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

/// 获取应用可执行文件所在目录
pub fn get_app_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.to_path_buf();
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

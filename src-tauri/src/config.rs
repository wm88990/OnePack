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
    /// 安装类型：exe（静默安装）| msi（静默安装）| green（绿色解压）
    #[serde(rename = "type")]
    pub install_type: String,
    /// 静默安装参数（exe/msi 类型使用）
    pub silent_args: String,
    /// 绿色软件主程序文件名（green 类型使用，用于创建快捷方式）
    #[serde(default)]
    pub main_exe: Option<String>,
    /// 绿色软件解压后的子目录名（green 类型使用，为空则解压到根目录）
    #[serde(default)]
    pub extract_subdir: Option<String>,
}

// ===== 默认配置 =====

/// 默认配置 YAML 内容
const DEFAULT_CONFIG_YAML: &str = r#"meta:
  name: "我的装机方案"
  version: "1.0.0"
  created: "2026-07-19"

categories:
  - id: system
    name: 系统工具
    icon: "⚙️"
  - id: browser
    name: 网络浏览
    icon: "🌐"
  - id: office
    name: 办公应用
    icon: "📄"
  - id: social
    name: 通讯社交
    icon: "💬"
  - id: media
    name: 影音娱乐
    icon: "🎵"
  - id: dev
    name: 开发工具
    icon: "🛠️"
  - id: green
    name: 绿色软件
    icon: "🍃"

packages:
  # ===== 系统工具 =====
  - id: "7zip"
    name: "7-Zip"
    icon: "📦"
    category: system
    description: "开源压缩解压工具，支持多种格式"
    version: "24.09"
    size: "1.5 MB"
    source:
      type: local
      path: "7z2409-x64.exe"
      hash_sha256: ""
    install:
      type: exe
      silent_args: "/S"

  - id: "winrar"
    name: "WinRAR"
    icon: "📚"
    category: system
    description: "强大的压缩文件管理器"
    version: "6.24"
    size: "3.4 MB"
    source:
      type: local
      path: "WinRAR-x64.exe"
      hash_sha256: ""
    install:
      type: exe
      silent_args: "/S"

  - id: "huorong"
    name: "火绒安全"
    icon: "🛡️"
    category: system
    description: "轻量级安全防护软件"
    version: "5.0.82"
    size: "32 MB"
    source:
      type: local
      path: "Huorong.exe"
      hash_sha256: ""
    install:
      type: exe
      silent_args: "/S"

  # ===== 网络浏览 =====
  - id: "chrome"
    name: "Google Chrome"
    icon: "🌐"
    category: browser
    description: "谷歌浏览器"
    version: "131.0"
    size: "95 MB"
    source:
      type: local
      path: "ChromeSetup.exe"
      hash_sha256: ""
    install:
      type: exe
      silent_args: "/silent /install"

  # ===== 通讯社交 =====
  - id: "wechat"
    name: "微信"
    icon: "💬"
    category: social
    description: "腾讯即时通讯工具"
    version: "3.9.12"
    size: "280 MB"
    source:
      type: local
      path: "WeChatSetup.exe"
      hash_sha256: ""
    install:
      type: exe
      silent_args: "/S /AutoInstall"

  - id: "qq"
    name: "QQ"
    icon: "🐧"
    category: social
    description: "腾讯即时通讯工具"
    version: "9.9.15"
    size: "320 MB"
    source:
      type: local
      path: "QQNTSetup.exe"
      hash_sha256: ""
    install:
      type: exe
      silent_args: "/S"

  # ===== 开发工具 =====
  - id: "vscode"
    name: "VS Code"
    icon: "💻"
    category: dev
    description: "微软轻量代码编辑器"
    version: "1.96"
    size: "98 MB"
    source:
      type: local
      path: "VSCodeSetup-x64.exe"
      hash_sha256: ""
    install:
      type: exe
      silent_args: "/VERYSILENT /MERGETASKS=!RUN /NORESTART"

  - id: "git"
    name: "Git"
    icon: "📁"
    category: dev
    description: "分布式版本控制系统"
    version: "2.47"
    size: "58 MB"
    source:
      type: local
      path: "Git-Setup.exe"
      hash_sha256: ""
    install:
      type: exe
      silent_args: "/VERYSILENT /NORESTART"

  # ===== 绿色软件 =====
  - id: "notepadpp"
    name: "Notepad++"
    icon: "📝"
    category: green
    description: "轻量级代码编辑器，免安装"
    version: "8.7"
    size: "4.8 MB"
    source:
      type: local
      path: "npp.8.7.portable.zip"
      hash_sha256: ""
    install:
      type: green
      silent_args: ""
      main_exe: "notepad++.exe"
      extract_subdir: "Notepad++"

  - id: "potplayer"
    name: "PotPlayer"
    icon: "🎬"
    category: green
    description: "高性能多媒体播放器，免安装"
    version: "1.7.22"
    size: "38 MB"
    source:
      type: local
      path: "PotPlayerSetup.zip"
      hash_sha256: ""
    install:
      type: green
      silent_args: ""
      main_exe: "PotPlayerMini64.exe"
      extract_subdir: "PotPlayer"

  - id: "everything"
    name: "Everything"
    icon: "🔍"
    category: green
    description: "极速文件搜索工具，免安装"
    version: "1.4.1"
    size: "1.8 MB"
    source:
      type: local
      path: "Everything-1.4.1.zip"
      hash_sha256: ""
    install:
      type: green
      silent_args: ""
      main_exe: "Everything.exe"
      extract_subdir: "Everything"

  - id: "sumatrapdf"
    name: "SumatraPDF"
    icon: "📖"
    category: green
    description: "轻量级 PDF 阅读器，免安装"
    version: "3.5.2"
    size: "6.5 MB"
    source:
      type: local
      path: "SumatraPDF-3.5.2.zip"
      hash_sha256: ""
    install:
      type: green
      silent_args: ""
      main_exe: "SumatraPDF.exe"
      extract_subdir: "SumatraPDF"

  - id: "irfanview"
    name: "IrfanView"
    icon: "🖼️"
    category: green
    description: "轻量级图片查看器，免安装"
    version: "4.70"
    size: "5 MB"
    source:
      type: local
      path: "iview470_plugins.zip"
      hash_sha256: ""
    install:
      type: green
      silent_args: ""
      main_exe: "i_view64.exe"
      extract_subdir: "IrfanView"
"#;

// ===== 配置加载 =====

/// 加载 config.yaml 配置文件，不存在则生成默认配置
pub fn load_config(app_dir: &Path) -> Result<Config, String> {
    let config_path = app_dir.join("config.yaml");

    // 如果配置文件不存在，生成默认配置
    if !config_path.exists() {
        generate_default_config(&config_path)?;
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let mut config: Config = serde_yaml::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 扫描 packages/ 目录，标记哪些包有本地文件
    let packages_dir = app_dir.join("packages");
    if packages_dir.exists() {
        for pkg in &mut config.packages {
            if pkg.source.source_type == "local" {
                if let Some(ref path) = pkg.source.path {
                    let full_path = packages_dir.join(path);
                    pkg.source.path = Some(full_path.exists().to_string());
                }
            }
        }
    }

    Ok(config)
}

/// 生成默认配置文件
pub fn generate_default_config(config_path: &Path) -> Result<(), String> {
    // 确保父目录存在
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    fs::write(config_path, DEFAULT_CONFIG_YAML)
        .map_err(|e| format!("写入默认配置失败: {}", e))?;

    Ok(())
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

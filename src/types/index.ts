// ===== 软件包类型定义 =====

export interface PackageItem {
  id: string
  name: string
  icon: string
  category: string
  description: string
  version: string
  size: string
  enabled: boolean              // 是否启用，禁用时跳过安装
  source: PackageSource
  install: PackageInstall
}

export interface PackageSource {
  type: 'local' | 'remote'
  path?: string        // local 模式下相对于 packages/ 的路径
  url?: string         // remote 模式下的下载地址
  hash_sha256?: string
}

export interface PackageInstall {
  type: 'exe' | 'msi' | 'green'
  silent_args: string
  manual_args?: string
  main_exe?: string         // 绿色软件主程序文件名
  extract_subdir?: string   // 绿色软件解压后的子目录名
  install_dir?: string      // 自定义安装目录
  dir_format?: string       // 目录参数格式（如 /D=、-d）
  create_shortcut?: boolean // 是否创建桌面快捷方式
  shortcut_name?: string    // 快捷方式名称
}

// ===== 分类类型 =====

export interface CategoryItem {
  id: string
  name: string
  icon: string
}

// ===== 配置文件类型 =====

export interface ConfigFile {
  meta: {
    name: string
    version: string
    created?: string
  }
  categories: CategoryItem[]
  packages: PackageItem[]
}

// ===== 安装状态 =====

export type InstallStatus = 'none' | 'installed' | 'outdated' | 'installing' | 'failed'

// ===== 安装进度事件 =====

export interface InstallProgressEvent {
  type: 'start' | 'progress' | 'success' | 'error' | 'complete'
  package_id: string
  package_name: string
  progress?: number      // 0-100
  current?: number       // 当前第几个
  total?: number         // 总共几个
  message?: string
  failed?: number        // complete 时失败的个数
}

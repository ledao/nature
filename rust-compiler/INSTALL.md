# Nature编译器Rust重写 - 安装指南

## 🚀 快速开始

### 1. 安装Rust

#### 方法一：使用rustup（推荐）
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### 方法二：使用包管理器
```bash
# Ubuntu/Debian
sudo apt install cargo rustc

# CentOS/RHEL
sudo yum install cargo rustc

# macOS
brew install rust
```

### 2. 验证安装
```bash
rustc --version
cargo --version
```

### 3. 构建项目
```bash
cd rust-compiler
cargo build
```

### 4. 运行测试
```bash
cargo test
```

### 5. 运行基准测试
```bash
cargo bench
```

## 📋 系统要求

- **Rust**: 1.70.0 或更高版本
- **LLVM**: 15.0 或更高版本（可选，用于代码生成）
- **内存**: 至少 2GB RAM
- **磁盘空间**: 至少 1GB 可用空间

## 🔧 依赖项

项目使用以下主要依赖：

- **inkwell**: LLVM Rust绑定
- **logos**: 高性能词法分析
- **nom**: 解析器组合子
- **thiserror**: 错误处理
- **serde**: 序列化支持
- **criterion**: 性能基准测试

## 🐛 故障排除

### 常见问题

1. **LLVM未找到**
   ```bash
   # Ubuntu/Debian
   sudo apt install llvm-15-dev
   
   # CentOS/RHEL
   sudo yum install llvm15-devel
   ```

2. **权限问题**
   ```bash
   chmod +x install.sh
   ```

3. **网络问题**
   ```bash
   # 设置Rust镜像
   export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
   export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
   ```

## 📚 开发环境

### 推荐IDE
- **VS Code** + Rust扩展
- **IntelliJ IDEA** + Rust插件
- **Vim/Neovim** + rust.vim

### 有用的工具
```bash
# 安装开发工具
rustup component add rustfmt clippy

# 代码格式化
cargo fmt

# 代码检查
cargo clippy

# 文档生成
cargo doc --open
```

## 🎯 下一步

安装完成后，您可以：

1. 查看示例程序：`examples/` 目录
2. 运行测试：`cargo test`
3. 查看文档：`cargo doc --open`
4. 开始开发：编辑 `src/` 目录中的文件

## 📞 支持

如果遇到问题，请：

1. 检查 [故障排除](#故障排除) 部分
2. 查看项目文档
3. 提交Issue到项目仓库

---

**注意**: 这是一个开发版本，某些功能可能尚未完全实现。

# Nature Rust编译器

一个用Rust实现的Nature编程语言编译器，使用LLVM作为后端。

## 🚀 快速开始

### 1. 环境初始化（首次使用）
```bash
./init.sh
```
这个脚本会：
- ✅ 检查Rust环境
- ✅ 检查LLVM 15工具链
- ✅ 检查链接器（GCC）
- ✅ 自动安装缺失的依赖
- ✅ 测试编译环境

### 2. 构建项目
```bash
cargo build --release
```

### 3. 运行示例
```bash
# 编译Nature源代码
./target/release/nrc build --input examples/basic.n

# 运行生成的可执行文件
./main
```

## 📁 项目结构

```
rust-compiler/
├── src/                    # 源代码
│   ├── lexer/             # 词法分析器
│   ├── parser/            # 语法分析器
│   ├── semantic/          # 语义分析器
│   ├── llvm_backend/      # LLVM后端
│   └── lib.rs             # 主库文件
├── examples/              # 示例文件
│   ├── basic.n           # 基础示例
│   └── test_builtins.n   # 内置函数测试
└── Cargo.toml            # 项目配置
```

## 🛠️ 系统要求

- **操作系统**: Linux (Ubuntu/Debian推荐)
- **Rust**: 1.70+
- **LLVM**: 15.x (自动检测)
- **GCC**: 9.0+ (作为链接器)

## 📋 支持的功能

### 语言特性
- ✅ 函数定义和调用
- ✅ 变量声明 (`var`, `let`)
- ✅ 基本数据类型 (`int`, `float`, `string`)
- ✅ 算术运算 (`+`, `-`, `*`, `/`)
- ✅ 控制流 (`if`, `while`, `for`)
- ✅ 返回语句 (`return`)

### 内置函数
- ✅ `println()` - 打印并换行
- ✅ `print()` - 打印不换行
- ✅ `len()` - 获取字符串长度

### 编译器特性
- ✅ LLVM IR生成
- ✅ 机器码生成
- ✅ 可执行文件生成
- ✅ 错误报告
- ✅ 类型检查

## 🧪 示例

### basic.n
```nature
fn add(int a, int b): int {
    return a + b;
}

fn main() {   
    println("hello nature\n");
    println("3 + 2 = ", add(3, 2));
}
```

### test_builtins.n
```nature
fn test_print() {
    print("Hello ");
    println("World!");
    println("Numbers: ", 42, " and ", 3.14);
}

fn test_len() {
    var text = "Hello Nature";
    var length = len(text);
    println("Length of '", text, "' is ", length);
}

fn main() {
    test_print();
    test_len();
}
```

## 🔧 开发

### 运行测试
```bash
cargo test
```

### 查看文档
```bash
cargo doc --open
```

### 调试构建
```bash
cargo build
```

## 🐛 故障排除

### 常见问题

1. **LLVM未找到**
   ```bash
   sudo apt install -y llvm-15-dev llvm-15-tools
   ```

2. **链接器错误**
   ```bash
   sudo apt install -y gcc
   ```

3. **编译错误**
   ```bash
   cargo clean
   cargo build --release
   ```

## 📄 许可证

本项目采用MIT许可证。

## 🤝 贡献

欢迎提交Issue和Pull Request！
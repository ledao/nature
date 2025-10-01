# 🚀 Nature Rust编译器快速开始指南

## 第一次使用

### 1. 环境初始化
```bash
./init.sh
```
这个脚本会：
- 检查并安装Rust、LLVM、GCC等必要工具
- 设置环境变量
- 测试编译环境
- 创建`.env`配置文件

### 2. 加载环境
```bash
source .env
```

### 3. 构建项目
```bash
cargo build --release
```

### 4. 运行示例
```bash
# 编译Nature代码
./target/release/nrc build --input examples/basic.n

# 运行生成的可执行文件
./main
```

## 日常使用

### 每次使用前
```bash
source .env  # 加载环境变量
```

### 编译Nature代码
```bash
./target/release/nrc build --input your_file.n
./main  # 运行生成的可执行文件
```

### 检查环境状态
```bash
./test_env.sh  # 检查环境是否正确配置
```

## 示例代码

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

## 故障排除

### 环境问题
```bash
./init.sh  # 重新初始化环境
```

### 编译问题
```bash
source .env  # 确保环境变量已加载
cargo clean  # 清理构建缓存
cargo build --release  # 重新构建
```

### 权限问题
```bash
chmod +x init.sh
chmod +x test_env.sh
```

## 支持的功能

✅ **语言特性**
- 函数定义和调用
- 变量声明 (`var`, `let`)
- 基本数据类型 (`int`, `float`, `string`)
- 算术运算
- 控制流
- 返回语句

✅ **内置函数**
- `println()` - 打印并换行
- `print()` - 打印不换行  
- `len()` - 获取字符串长度

✅ **编译器特性**
- LLVM IR生成
- 机器码生成
- 可执行文件生成
- 错误报告
- 类型检查

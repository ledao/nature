#!/bin/bash

# Nature编译器构建脚本
# 设置LLVM环境变量并构建项目

set -e

echo "🔨 构建Nature编译器..."

# 设置LLVM环境变量
export LLVM_SYS_150_PREFIX=/usr
export LLVM_CONFIG_PATH=/usr/bin/llvm-config-15

# 检查LLVM配置
echo "📋 检查LLVM配置..."
if [ -f "$LLVM_CONFIG_PATH" ]; then
    echo "✅ LLVM配置路径: $LLVM_CONFIG_PATH"
    echo "   版本: $($LLVM_CONFIG_PATH --version)"
else
    echo "❌ 找不到LLVM配置: $LLVM_CONFIG_PATH"
    exit 1
fi

# 检查LLVM库路径
LLVM_LIB_PATH="/usr/lib/llvm-15/lib"
if [ -d "$LLVM_LIB_PATH" ]; then
    echo "✅ LLVM库路径: $LLVM_LIB_PATH"
    export LLVM_LIB_PATH
else
    echo "❌ 找不到LLVM库路径: $LLVM_LIB_PATH"
    exit 1
fi

# 设置链接器标志
export RUSTFLAGS="-L $LLVM_LIB_PATH"

echo "🚀 开始构建..."
cargo build --verbose

echo "✅ 构建完成！"
echo ""
echo "🧪 运行测试..."
cargo test

echo "✅ 所有测试通过！"
echo ""
echo "🎉 Nature编译器构建成功！"
echo ""
echo "📁 可执行文件位置: target/debug/nrc"
echo "📚 查看文档: cargo doc --open"
echo "🏃 运行示例: ./target/debug/nrc build --input examples/basic.n"

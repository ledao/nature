#!/bin/bash

# Nature Rust编译器环境初始化脚本
# 检查并安装必要的编译环境

set -e

echo "🚀 Nature Rust编译器环境初始化"
echo "=================================="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查函数
check_command() {
    local cmd=$1
    local package=$2
    local install_cmd=$3
    
    if command -v "$cmd" &> /dev/null; then
        echo -e "${GREEN}✅ $cmd 已安装${NC}"
        return 0
    else
        echo -e "${RED}❌ $cmd 未安装${NC}"
        echo -e "${YELLOW}需要安装: $package${NC}"
        echo -e "${BLUE}安装命令: $install_cmd${NC}"
        return 1
    fi
}

# 安装函数
install_package() {
    local package=$1
    local install_cmd=$2
    
    echo ""
    echo -e "${YELLOW}是否安装 $package? (Y/n)${NC}"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]] || [[ -z "$response" ]]; then
        echo -e "${BLUE}正在安装 $package...${NC}"
        eval "$install_cmd"
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✅ $package 安装成功${NC}"
        else
            echo -e "${RED}❌ $package 安装失败${NC}"
            exit 1
        fi
    else
        echo -e "${YELLOW}跳过 $package 安装${NC}"
        return 1
    fi
}

# 检查Rust
echo "📋 检查Rust环境..."
if check_command "rustc" "Rust" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"; then
    echo -e "${GREEN}   Rust版本: $(rustc --version)${NC}"
else
    install_package "Rust" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
    echo -e "${BLUE}请重新加载shell环境: source ~/.cargo/env${NC}"
fi

echo ""

# 检查LLVM
echo "📋 检查LLVM环境..."
llvm_installed=false

if check_command "llvm-config-15" "LLVM 15" "sudo apt update && sudo apt install -y llvm-15-dev llvm-15-tools"; then
    echo -e "${GREEN}   LLVM版本: $(llvm-config-15 --version)${NC}"
    llvm_installed=true
elif check_command "llvm-config" "LLVM" "sudo apt update && sudo apt install -y llvm-dev llvm-tools"; then
    echo -e "${GREEN}   LLVM版本: $(llvm-config --version)${NC}"
    llvm_installed=true
fi

if [ "$llvm_installed" = false ]; then
    install_package "LLVM 15" "sudo apt update && sudo apt install -y llvm-15-dev llvm-15-tools"
fi

echo ""

# 检查llc
echo "📋 检查LLVM编译器..."
if check_command "llc-15" "LLVM编译器" "sudo apt install -y llvm-15-tools"; then
    echo -e "${GREEN}   llc-15版本: $(llc-15 --version | head -n1)${NC}"
else
    install_package "LLVM编译器" "sudo apt install -y llvm-15-tools"
fi

echo ""

# 检查链接器
echo "📋 检查链接器..."
linker_installed=false

if check_command "gcc" "GCC编译器" "sudo apt update && sudo apt install -y gcc"; then
    echo -e "${GREEN}   GCC版本: $(gcc --version | head -n1)${NC}"
    linker_installed=true
fi

if [ "$linker_installed" = false ]; then
    install_package "GCC编译器" "sudo apt update && sudo apt install -y gcc"
fi

echo ""

# 检查构建工具
echo "📋 检查构建工具..."
if check_command "cargo" "Cargo" "已包含在Rust安装中"; then
    echo -e "${GREEN}   Cargo版本: $(cargo --version)${NC}"
else
    echo -e "${RED}❌ Cargo未找到，请重新安装Rust${NC}"
    exit 1
fi

echo ""

# 测试编译
echo "🧪 测试编译环境..."
if cargo build --release > /dev/null 2>&1; then
    echo -e "${GREEN}✅ 编译测试成功${NC}"
else
    echo -e "${RED}❌ 编译测试失败${NC}"
    echo -e "${YELLOW}请检查错误信息并重新运行此脚本${NC}"
    exit 1
fi

echo ""

# 测试编译器
echo "🧪 测试Nature编译器..."
if ./target/release/nrc build --input examples/basic.n > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Nature编译器测试成功${NC}"
    if ./main > /dev/null 2>&1; then
        echo -e "${GREEN}✅ 生成的可执行文件运行正常${NC}"
    else
        echo -e "${YELLOW}⚠️  生成的可执行文件运行异常${NC}"
    fi
else
    echo -e "${RED}❌ Nature编译器测试失败${NC}"
    exit 1
fi

echo ""
echo "🎉 环境初始化完成！"
echo "=================================="
echo ""
echo "📚 使用方法:"
echo "  1. 构建项目: cargo build --release"
echo "  2. 运行示例: ./target/release/nrc build --input examples/basic.n"
echo "  3. 执行程序: ./main"
echo ""
echo "💡 提示: 现在可以直接使用编译器了！"

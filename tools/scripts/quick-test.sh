#!/bin/bash
# 快速测试脚本
# 用于开发时快速验证修改

set -e

echo "=== 明朝修仙 RPG - 快速测试 ==="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

cd "$(dirname "$0")/../.."

echo -e "${YELLOW}[1/4]${NC} 检查代码格式..."
xmake format-check || {
    echo -e "${RED}格式检查失败，运行 'xmake format' 修复${NC}"
    exit 1
}

echo -e "${YELLOW}[2/4]${NC} 运行静态检查..."
xmake check || {
    echo -e "${RED}静态检查失败${NC}"
    exit 1
}

echo -e "${YELLOW}[3/4]${NC} 构建 Debug 版本..."
cd engine
cargo build --quiet 2>&1 | grep -v "warning:" || true
cd ..

echo -e "${YELLOW}[4/4]${NC} 运行测试..."
cd engine
cargo test --quiet 2>&1 | tail -20
cd ..

echo -e "${GREEN}✓ 快速测试通过！${NC}"
echo ""
echo "运行 'xmake run' 启动游戏"

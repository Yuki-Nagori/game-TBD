#!/bin/bash
# 彻底清理脚本
# 删除所有构建产物和缓存

echo "=== 明朝修仙 RPG - 彻底清理 ==="
echo "⚠️  这将删除所有构建产物！"
echo ""

read -p "确认清理? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "已取消"
    exit 0
fi

cd "$(dirname "$0")/../.."

echo "[1/5] 清理 xmake 构建..."
xmake clean 2>/dev/null || true
rm -rf build/ .xmake/

echo "[2/5] 清理 Rust 构建..."
cd engine
cargo clean 2>/dev/null || true
cd ..

echo "[3/5] 清理缓存..."
rm -rf engine/target/
rm -rf dist/
rm -rf build/

echo "[4/5] 清理日志..."
rm -rf logs/
rm -f *.log

echo "[5/5] 清理临时文件..."
find . -name "*.tmp" -delete 2>/dev/null || true
find . -name ".DS_Store" -delete 2>/dev/null || true
find . -name "*~" -delete 2>/dev/null || true

echo ""
echo "✓ 清理完成"
echo "运行 'xmake b' 重新构建"

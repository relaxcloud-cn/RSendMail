#!/bin/bash

# 测试附件修复的脚本
echo "=== 测试附件修复效果 ==="

# 进入项目目录
cd rsendmail

# 构建项目
echo "构建项目..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ 构建失败"
    exit 1
fi

echo "✅ 构建成功"

# 显示可用的测试EML文件
echo ""
echo "可用的测试EML文件："
ls -la ../test_data/*.eml

echo ""
echo "=== 测试方案说明 ==="
echo "1. 不使用 --keep-headers 参数（之前会丢失附件，现在应该保留）"
echo "2. 使用 --keep-headers 参数（一直正常，用作对比）"
echo ""

echo "请手动运行以下命令进行测试："
echo ""
echo "# 测试默认模式（修复后应该保留附件）："
echo "cargo run -- --smtp-server your_smtp_server --from test@example.com --to recipient@example.com --dir ../test_data"
echo ""
echo "# 测试 keep-headers 模式（应该和之前一样正常）："
echo "cargo run -- --smtp-server your_smtp_server --from test@example.com --to recipient@example.com --dir ../test_data --keep-headers"
echo ""
echo "对比两种模式发送的邮件，确认附件是否都被保留。" 
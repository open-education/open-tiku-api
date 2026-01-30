#!/bin/sh

#
# 生产环境包构建脚本
# 仅构建生产压缩包, 需要手动上传到目标机器或者使用其它脚本上传到指定位置后再使用
#

set -e

echo "开始编译项目生产环境可执行程序二进制压缩文件..."
echo "其它环境无需运行该脚本, 直接使用 IDE 运行或者简单执行 cargo run 即可"
echo "系统信息: $(uname -a)"
echo "当前用户: $(whoami)"

if command -v cargo >/dev/null 2>&1; then
    echo "Rust 已安装 (通过 cargo 命令)"
    cargo --version
else
    echo "Rust 未安装 (未找到 cargo 命令)"
    exit 1
fi

echo "开始编译..."
if cargo build --release; then
    echo "编译成功"
else
    echo "编译失败, 请查看上面的错误输出信息调整"
    exit 1
fi

sleep 5

TARGET_PATH="target/release/open-tiku-api"
echo "编译结果文件位置: ${TARGET_PATH}"

echo "检查编译文件..."
if [ -f "${TARGET_PATH}" ] && [ -x "${TARGET_PATH}" ]; then
    echo "编译后目标文件存在且可执行"
else
    echo "编译后目标文件不存在或不可执行"
    exit 1
fi

echo "检查文件大小..."
if [ -f "${TARGET_PATH}" ]; then
    size_bytes=$(wc -c < "${TARGET_PATH}" 2>/dev/null || echo "0")
else
    size_bytes="0"
fi

if [ -z "$size_bytes" ] || [ "$size_bytes" -eq 0 ]; then
    echo "无法获取文件大小或文件为空: ${TARGET_PATH}"
    exit 1
fi

echo "文件大小: ${size_bytes} kb"
if [ "$size_bytes" -lt 1024 ]; then
    echo "检查失败: 编译后的目标文件较小: ${size_bytes} 字节, 不足1kb"
    exit 1
fi

echo "开始压缩..."
if tar -czf target/release/open-tiku-api.tgz -C "target/release" "open-tiku-api"; then
    echo "压缩成功"
    echo "生成文件位置: target/release/open-tiku-api.tgz, 在目标文件使用类似命令解压缩即可: tar xvf open-tiku-api.tgz"
else
    echo "压缩失败"
    exit 1
fi

echo "全部完成"

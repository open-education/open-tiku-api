#!/bin/bash

set -e

# 服务器如果访问 GitHub 无法等待, 可以考虑通过本地上传
# 用法: ./upload.sh <应用名> <版本号>
# 示例: ./upload.sh open-tiku-api v0.0.26
# 功能: 本地下载 GitHub Release .tgz 并通过 scp 上传到远程服务器

# 远程服务器配置
REMOTE_HOST="your-server-ip"
REMOTE_USER="your-username"
REMOTE_PATH="/path/to/destination/"
SSH_PORT="22"

if [ $# -ne 2 ]; then
    echo "用法: $0 <应用名> <版本号>"
    exit 1
fi

APP="$1"
VERSION="$2"
URL="https://github.com/open-education/${APP}/releases/download/${VERSION}/${APP}.tgz"
FILE="${APP}.tgz"

echo "下载: ${URL}"
echo "本地保存: ${LOCAL_FILE}"

curl -f -L -o "$FILE" "$URL"
echo "下载完成: $FILE"

echo "上传至 ${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_PATH}"

scp -P "$SSH_PORT" "$FILE" "$REMOTE_USER@$REMOTE_HOST:$REMOTE_PATH"
echo "上传完成"

rm -f "$FILE"

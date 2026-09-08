#!/bin/bash
#
# Debian 12 手动备份脚本（带日期后缀）
# 功能：备份 /var/www/meta 目录和 PostgreSQL 数据库
# 备份文件存放于脚本运行目录，文件名为 meta-YYYYMMDD.tgz 和 postgresql-data-YYYYMMDD.tgz
# meta.tgz 解压后直接为 meta 目录（不包含 /var/www 路径）
# 执行时如需要密码，会提示手动输入

set -euo pipefail

# 获取当前日期（年月日）
BACKUP_DATE=$(date +%Y%m%d)

# 定义备份目录（脚本所在当前目录）
BACKUP_DIR="$(pwd)"

# 定义备份目标（带日期后缀）
META_SRC="/var/www/meta"
META_ARCHIVE="${BACKUP_DIR}/meta-${BACKUP_DATE}.tgz"

# PostgreSQL 数据库相关（可调整）
PG_DB="open_tiku_test"          # 请替换为实际数据库名
PG_USER="tiku_rw"             # 请替换为实际用户名
PG_ARCHIVE="${BACKUP_DIR}/postgresql-data-${BACKUP_DATE}.tgz"
TEMP_PG_DUMP="/tmp/postgresql-backup-$$.sql"

# 颜色输出（便于阅读）
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== 手动备份脚本开始 ===${NC}"
echo "备份日期: ${BACKUP_DATE}"
echo "备份目录: ${BACKUP_DIR}"

# ------------------------------------------------------------
# 1. 备份 /var/www/meta 目录 (打包为 meta-日期.tgz)
# ------------------------------------------------------------
if [ -d "${META_SRC}" ]; then
    echo -e "${YELLOW}正在打包 ${META_SRC} ...${NC}"
    # 使用 tar 的 -C 选项切换工作目录，使解压后直接为 meta 目录
    if tar -czf "${META_ARCHIVE}" -C "$(dirname "${META_SRC}")" "$(basename "${META_SRC}")"; then
        echo -e "${GREEN}✓ meta 目录备份成功: ${META_ARCHIVE}${NC}"
    else
        echo -e "${RED}✗ meta 目录备份失败，请检查权限或路径${NC}"
        exit 1
    fi
else
    echo -e "${RED}✗ 源目录 ${META_SRC} 不存在，跳过 meta 备份${NC}"
fi

# ------------------------------------------------------------
# 2. 备份 PostgreSQL 数据库 (导出为 SQL 后打包)
# ------------------------------------------------------------
echo -e "${YELLOW}准备备份 PostgreSQL 数据库...${NC}"

# 检查 pg_dump 是否存在
if ! command -v pg_dump &> /dev/null; then
    echo -e "${RED}✗ pg_dump 命令未找到，请确认 postgresql-client 已安装${NC}"
    exit 1
fi

# 提示数据库信息
echo "数据库名: ${PG_DB}"
echo "用户名: ${PG_USER}"
echo "如果需要密码，请在下方提示时手动输入。"

# 执行 pg_dump，密码由用户交互输入（如需要）
if pg_dump -U "${PG_USER}" -d "${PG_DB}" -h 127.0.0.1 > "${TEMP_PG_DUMP}" 2>/dev/null; then
    echo -e "${GREEN}✓ 数据库导出成功${NC}"
    # 打包 SQL 文件为 postgresql-data-日期.tgz
    if tar -czf "${PG_ARCHIVE}" -C "$(dirname "${TEMP_PG_DUMP}")" "$(basename "${TEMP_PG_DUMP}")"; then
        echo -e "${GREEN}✓ 数据库备份打包成功: ${PG_ARCHIVE}${NC}"
        # 清理临时文件
        rm -f "${TEMP_PG_DUMP}"
    else
        echo -e "${RED}✗ 数据库打包失败${NC}"
        rm -f "${TEMP_PG_DUMP}"
        exit 1
    fi
else
    echo -e "${RED}✗ 数据库导出失败，请检查数据库名、用户名或密码是否正确${NC}"
    # 清理可能残留的临时文件
    rm -f "${TEMP_PG_DUMP}"
    exit 1
fi

# ------------------------------------------------------------
# 3. 完成提示
# ------------------------------------------------------------
echo -e "${GREEN}=== 备份完成 ===${NC}"
echo "生成文件列表:"
ls -lh "${META_ARCHIVE}" "${PG_ARCHIVE}" 2>/dev/null || true

exit 0
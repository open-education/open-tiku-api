#!/bin/sh

# 清除 Sqlite 过期缓存数据
# crontab 命令
# 每日凌晨3点30分钟
# 30 3 * * * /bin/bash /xxx/cache_clean.sh >> /xxx/open-tiku-api/log/sh.log 2>&1

# 使用上海时区
export TZ="Asia/Shanghai"

# sqlite 数据库文件绝对路径
DB_PATH="/var/www/meta/cache/open_tiku_api_sqlite.db"

# 直接使用系统命令删除, 并回收表空间
sqlite3 "$DB_PATH" "DELETE FROM kv_cache WHERE expires_at <= strftime('%s', 'now'); VACUUM;"

#!/bin/sh

set -e

# 配置
APP_NAME="open-tiku-api"
META_PATH=""
PORT=8082
LOG_DIR="log"
PID_FILE="${APP_NAME}.pid"

# 显示帮助信息
show_help() {
    echo "使用方法: sh $0 {start|stop|restart|status} [选项]"
    echo ""
    echo "命令:"
    echo "  start     启动应用程序"
    echo "  stop      停止应用程序"
    echo "  restart   重启应用程序"
    echo "  status    查看应用程序状态"
    echo ""
    echo "选项:"
    echo "  --version <版本号>  指定部署版本, 版本号为空或者不提供直接操作现有文件, 如需下载文件请提供对应的版本号"
    echo "  --meta-path <路径>  指定 meta-path 参数 (默认: $META_PATH)"
    echo "  --port <端口>       指定 port 参数 (默认: $PORT)"
    echo "  --help             显示帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 start -v 2.1.0          # 启动版本 2.1.0"
    echo "  $0 stop                    # 停止应用程序"
    echo "  $0 restart -v 1.5.0        # 重启并升级到版本 1.5.0"
    echo "  $0 status                  # 查看状态"
    echo "  $0 sh deploy.sh restart -p 8082 -v v0.0.1-beta -m /home/.../open-tiku-meta # 常用完整命令名称, 端口因为要配置 nginx 代理转发调整需要对应调整 nginx 配置"
}

# 检查进程是否运行
check_pid() {
    _pid=""
    if [ -f "$PID_FILE" ]; then
        _pid=$(cat "$PID_FILE")
        if kill -0 "$_pid" 2>/dev/null; then
            echo "$_pid"
            return 0
        else
            # 进程不存在，清理 PID 文件
            rm -f "$PID_FILE"
            echo ""
            return 1
        fi
    fi
    # 没有进程文件返回空
    echo ""
}

# 显示状态
status() {
    _pid=""
    _pid=$(check_pid)

    echo "应用程序: $APP_NAME"
    echo "=========================="

    if [ -n "$_pid" ]; then
        echo "状态: 运行中"
        echo "进程ID: $_pid"

        # 获取运行时间
        _etime=$(ps -o etime= -p "$_pid" 2>/dev/null || echo "未知")
        echo "运行时间: $_etime"

        # 尝试获取可执行文件路径
        if [ -f "/proc/$_pid/exe" ]; then
            _app_exec=$(readlink -f "/proc/$_pid/exe" 2>/dev/null || echo "")
            if [ -n "$_app_exec" ] && [ -x "$_app_exec" ]; then
                echo "可执行文件: $_app_exec"
                # 尝试获取版本信息
                if "$_app_exec" --version 2>/dev/null | head -1; then
                    true
                fi
            fi
        fi

        # 显示内存使用
        _memory=$(ps -o rss= -p "$_pid" 2>/dev/null)
        if [ -n "$_memory" ]; then
            _memory_mb=$(echo "$_memory" | awk '{printf "%.1f MB", $1/1024}')
            echo "内存使用: $_memory_mb"
        fi
    else
        echo "状态: 未运行"
    fi
}

# 停止应用程序
stop_app() {
    _pid=""

    echo "停止应用程序..."

    _pid=$(check_pid)
    if [ -n "$_pid" ]; then
        echo "找到运行进程: $_pid"

        # 尝试优雅停止
        echo "发送终止信号..."
        kill "$_pid"

        # 等待进程结束
        _wait_time=0
        _max_wait=10

        while [ "$_wait_time" -lt "$_max_wait" ]; do
            if ! kill -0 "$_pid" 2>/dev/null; then
                echo "应用程序已正常停止"
                rm -f "$PID_FILE"
                return 0
            fi
            sleep 1
            _wait_time=$((_wait_time + 1))
            echo "等待进程结束... ($_wait_time/$_max_wait 秒)"
        done

        # 强制停止
        echo "应用程序未响应，强制停止..."
        kill -9 "$_pid" 2>/dev/null

        # 确认进程已结束
        sleep 1
        if kill -0 "$_pid" 2>/dev/null; then
            echo "错误: 无法停止进程 $_pid"
            return 1
        else
            echo "应用程序已强制停止"
            rm -f "$PID_FILE"
            return 0
        fi
    else
        echo "应用程序未运行"
        return 0
    fi
}

# 准备应用程序
prepare_app() {
    _version="$1"
    # 版本号为空则认为不寻找新文件, 直接操作现有文件
    # 方法2：检查并去除首尾空格
    _version_trimmed=$(echo "$_version" | xargs)
    if [ -z "$_version_trimmed" ]; then
        echo "版本号为空或者不提供, 直接操作现有文件, 如需下载文件请提供对应的版本号"
        return 0
    fi

    _app_file="${APP_NAME}"
    # https://github.com/open-education/open-tiku-api/releases/download/v0.0.1-beta/open-tiku-api.tgz
    _url="https://github.com/open-education/open-tiku-api/releases/download/${_version}/open-tiku-api.tgz"

    echo "准备版本 $_version..."
    echo "下载地址: $_url"

    # 清理旧文件
    rm -rf "${_app_file}.tgz" "${APP_NAME}"

    # 下载
    if command -v curl >/dev/null 2>&1; then
        if ! curl -f -L -o "${_app_file}.tgz" "$_url"; then
            echo "错误: 下载失败"
            exit 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -q -O "${_app_file}.tgz" "$_url"; then
            echo "错误: 下载失败"
            exit 1
        fi
    else
        echo "错误: 需要 curl 或 wget"
        exit 1
    fi

    # 检查文件
    if [ ! -s "${_app_file}.tgz" ]; then
        echo "错误: 下载的文件为空"
        exit 1
    fi

    # 解压
    echo "解压..."
    if ! tar -xzf "${_app_file}.tgz"; then
        echo "错误: 解压失败"
        exit 1
    fi

    # 查找可执行文件
    if [ -z "${APP_NAME}" ]; then
        echo "错误: 未找到可执行文件"
        echo "解压内容:"
        exit 1
    fi

    chmod +x "${APP_NAME}"
    echo "找到可执行文件: ${APP_NAME}"
}

# 启动应用程序
start_app() {
    _version="$1"

    echo "启动应用程序 (版本: $_version)..."

    # 准备应用程序
    prepare_app "$_version"

    # 检查是否已在运行
    _pid=""
    _pid=$(check_pid)
    if [ -n "$_pid" ]; then
        echo "应用程序已在运行 (PID: $_pid)"
        echo "尝试停止历史正在运行中的应用程序"
        stop_app
    fi

    # 准备日志目录
    mkdir -p "$LOG_DIR"
    _log_file="$LOG_DIR/${APP_NAME}_$(date '+%Y%m%d_%H%M%S').log"

    # 启动
    echo "启动命令: $APP_NAME --port=${PORT} --meta-path=$META_PATH"
    echo "日志文件: $_log_file"

    # 使用 nohup 启动
    nohup "./$APP_NAME" "--port=${PORT}" "--meta-path=${META_PATH}" > "$_log_file" 2>&1 &
    _app_pid=$!

    # 保存 PID
    echo "$_app_pid" > "$PID_FILE"

    # 验证启动
    echo "等待启动..."
    sleep 3

    if kill -0 "$_app_pid" 2>/dev/null; then
        echo "启动成功"
        echo "进程ID: $_app_pid"
        echo "版本: $_version"
        echo "日志: $_log_file"

        # 显示初始日志
        echo "--- 初始日志 ---"
        tail -5 "$_log_file" 2>/dev/null || echo "(日志文件为空)"
        return 0
    else
        echo "启动失败"
        echo "查看错误日志:"
        tail -10 "$_log_file" 2>/dev/null || echo "(无错误日志)"
        rm -f "$PID_FILE"
        return 1
    fi
}

# 重启应用程序
restart_app() {
    _version="$1"

    echo "重启应用程序..."

    # 先停止
    if ! stop_app; then
        echo "警告: 停止现有进程时遇到问题，继续尝试重启..."
    fi

    # 等待一下确保进程完全停止
    sleep 2

    # 再启动
    if start_app "$_version"; then
        echo "重启成功"
        return 0
    else
        echo "重启失败"
        return 1
    fi
}

# 解析参数
ACTION=""
VERSION=""

while [ $# -gt 0 ]; do
    case "$1" in
        start|stop|restart|status)
            ACTION="$1"
            shift
            ;;
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -m|--meta-path)
            META_PATH="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo "错误: 未知参数 $1"
            show_help
            exit 1
            ;;
    esac
done

# 根据命令执行相应操作
case "$ACTION" in
    start)
        start_app "$VERSION"
        ;;
    stop)
        stop_app
        ;;
    restart)
        restart_app "$VERSION"
        ;;
    status)
        status
        ;;
    *)
        echo "错误: 未知命令 $ACTION"
        show_help
        exit 1
        ;;
esac

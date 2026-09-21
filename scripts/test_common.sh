#!/usr/bin/env bash
# 测试公共库：被 scripts/test_mock_*.sh 引用，提供输出、服务启停、日志截取与断言等公共能力。
#
# 使用方式：
#     source "$(dirname "$0")/test_common.sh"
#
# 注意：本文件不直接执行，只提供函数与变量。

# ---------------------------------------------------------------------------
# 路径与常量
# ---------------------------------------------------------------------------

TEST_COMMON_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$TEST_COMMON_DIR/.." && pwd)"

# 网关与 mock 的监听地址。
GATEWAY_PORT=26380
MOCK_PORT=26383
GATEWAY_URL="http://127.0.0.1:$GATEWAY_PORT"
MOCK_URL="http://127.0.0.1:$MOCK_PORT"

# 网关运行期日志（application_directory 在 debug 下指向 target/debug/runtime）。
GATEWAY_LOG="$ROOT_DIR/target/debug/runtime/tmp/logs/debug.log"

# 测试过程产物目录。
TEST_OUT_DIR="$ROOT_DIR/target/test-mock"
MOCK_LOG="$TEST_OUT_DIR/mock.log"
GATEWAY_STDOUT="$TEST_OUT_DIR/gateway.out"
PID_FILE="$TEST_OUT_DIR/pids"

# 日志标记：网关与 mock 输出的关键行前缀。
MARK_GATEWAY="[MOCKTEST]"
MARK_MOCK="★★★"

# 场景脚本默认使用的测试模型。
TEST_MODEL="mock-chat"

# 断言计数。
PASS_COUNT=0
FAIL_COUNT=0

# ---------------------------------------------------------------------------
# 输出
# ---------------------------------------------------------------------------

if [ -t 1 ]; then
    C_RESET=$'\033[0m'
    C_RED=$'\033[31m'
    C_GREEN=$'\033[32m'
    C_YELLOW=$'\033[33m'
    C_BLUE=$'\033[36m'
    C_BOLD=$'\033[1m'
else
    C_RESET=""
    C_RED=""
    C_GREEN=""
    C_YELLOW=""
    C_BLUE=""
    C_BOLD=""
fi

title() { printf '\n%s=== %s ===%s\n' "$C_BOLD$C_BLUE" "$*" "$C_RESET"; }
info() { printf '%s[信息]%s %s\n' "$C_BLUE" "$C_RESET" "$*"; }
warn() { printf '%s[注意]%s %s\n' "$C_YELLOW" "$C_RESET" "$*"; }
ok() { printf '%s[通过]%s %s\n' "$C_GREEN" "$C_RESET" "$*"; PASS_COUNT=$((PASS_COUNT + 1)); }
fail() { printf '%s[失败]%s %s\n' "$C_RED" "$C_RESET" "$*"; FAIL_COUNT=$((FAIL_COUNT + 1)); }
detail() { printf '    %s\n' "$*"; }

# 输出一段关键内容，最多 20 行。
dump() {
    local label="$1"
    shift
    printf '    %s%s%s\n' "$C_BOLD" "$label" "$C_RESET"
    if [ $# -eq 0 ]; then
        printf '    (无)\n'
        return
    fi
    printf '%s\n' "$@" | head -20 | sed 's/^/    /'
}

# ---------------------------------------------------------------------------
# 断言
# ---------------------------------------------------------------------------

# 断言文本包含指定内容。
assert_contains() {
    local text="$1" needle="$2" label="$3"
    if printf '%s' "$text" | grep -qF -- "$needle"; then
        ok "$label（找到：$needle）"
        return 0
    fi
    fail "$label（未找到：$needle）"
    return 1
}

# 断言文本不包含指定内容。
assert_not_contains() {
    local text="$1" needle="$2" label="$3"
    if printf '%s' "$text" | grep -qF -- "$needle"; then
        fail "$label（不应出现：$needle）"
        return 1
    fi
    ok "$label（确认无：$needle）"
    return 0
}

# 打印本脚本的断言结果；返回非 0 表示存在失败。
finish_report() {
    local name="$1"
    title "$name 结果"
    printf '通过 %s%d%s 项，失败 %s%d%s 项\n' \
        "$C_GREEN" "$PASS_COUNT" "$C_RESET" "$C_RED" "$FAIL_COUNT" "$C_RESET"
    if [ "$FAIL_COUNT" -gt 0 ]; then
        return 1
    fi
    return 0
}

# ---------------------------------------------------------------------------
# 端口与服务
# ---------------------------------------------------------------------------

# 判断端口是否处于监听状态。
port_listening() {
    local port="$1"
    netstat -ano 2>/dev/null | grep -q ":$port .*LISTENING"
}

# 等待端口进入监听状态；超时返回非 0。
wait_for_port() {
    local port="$1" timeout="${2:-30}" waited=0
    while [ "$waited" -lt "$timeout" ]; do
        if port_listening "$port"; then
            return 0
        fi
        sleep 1
        waited=$((waited + 1))
    done
    return 1
}

# 取监听指定端口的进程号。
port_pid() {
    local port="$1"
    netstat -ano 2>/dev/null | grep ":$port .*LISTENING" | awk '{print $5}' | head -1
}

# 结束监听指定端口的进程。
stop_port() {
    local port="$1" name="$2" pid
    pid="$(port_pid "$port")"
    if [ -z "$pid" ]; then
        return 0
    fi
    info "停止 $name（端口 $port，进程 $pid）"
    taskkill //PID "$pid" //T //F >/dev/null 2>&1 || kill -9 "$pid" >/dev/null 2>&1
    local waited=0
    while [ "$waited" -lt 10 ] && port_listening "$port"; do
        sleep 1
        waited=$((waited + 1))
    done
}

# ---------------------------------------------------------------------------
# 日志截取
# ---------------------------------------------------------------------------

# 记录网关日志当前字节偏移，后续用 gateway_log_since_mark 取新增内容。
mark_gateway_log() {
    GW_LOG_MARK=0
    if [ -f "$GATEWAY_LOG" ]; then
        GW_LOG_MARK="$(wc -c < "$GATEWAY_LOG" | tr -d ' ')"
    fi
}

# 取网关日志自标记点之后的新增内容。
gateway_log_since_mark() {
    if [ ! -f "$GATEWAY_LOG" ]; then
        return 0
    fi
    tail -c "+$((GW_LOG_MARK + 1))" "$GATEWAY_LOG" 2>/dev/null
}

# 记录 mock 日志当前字节偏移。
mark_mock_log() {
    MOCK_LOG_MARK=0
    if [ -f "$MOCK_LOG" ]; then
        MOCK_LOG_MARK="$(wc -c < "$MOCK_LOG" | tr -d ' ')"
    fi
}

# 取 mock 日志自标记点之后的新增内容。
mock_log_since_mark() {
    if [ ! -f "$MOCK_LOG" ]; then
        return 0
    fi
    tail -c "+$((MOCK_LOG_MARK + 1))" "$MOCK_LOG" 2>/dev/null
}

# 取网关日志中带标记的行。
gateway_marked_lines() {
    printf '%s' "$1" | grep -F "$MARK_GATEWAY" | sed 's/.*\[MOCKTEST\]/[MOCKTEST]/' || true
}

# 取 mock 日志中带标记的行。
mock_marked_lines() {
    printf '%s' "$1" | grep -F "$MARK_MOCK" || true
}
# ---------------------------------------------------------------------------
# 时间比对
# ---------------------------------------------------------------------------

# 当前时钟，形如 17:30:38.962。
now_clock() {
    date +%H:%M:%S.%3N
}

# 把 HH:MM:SS.mmm 转成当日秒数（含小数）。
clock_to_seconds() {
    printf '%s' "$1" | awk -F'[:.]' '{ printf "%.3f", $1*3600 + $2*60 + $3 + ($4/1000) }'
}

# 计算两个时钟字符串之差（后者减前者），单位秒。
clock_diff() {
    local from to
    from="$(clock_to_seconds "$1")"
    to="$(clock_to_seconds "$2")"
    awk -v a="$from" -v b="$to" 'BEGIN { printf "%.3f", b - a }'
}

# 取文本中首个匹配行的时钟前缀；无匹配时输出空。
first_clock_of() {
    local text="$1" needle="$2" line
    line="$(printf '%s' "$text" | grep -F -- "$needle" | head -1)"
    if [ -z "$line" ]; then
        return 0
    fi
    # mock 日志行首为 "[HH:MM:SS.mmm]"，网关日志为 "HH:MM:SS.mmm"。
    printf '%s' "$line" | grep -oE '[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}' | head -1
}

# ---------------------------------------------------------------------------
# 请求
# ---------------------------------------------------------------------------

# 发起一次可能被中途取消的流式/非流式请求。
#
# 参数：$1 场景名，$2 请求体 JSON，$3 最大等待秒数，其余参数为附加 curl 头。
# 客户端在达到超时或被 head 截断时主动断开，用于模拟取消。
call_gateway_disconnect() {
    local name="$1" body="$2" max_seconds="$3"
    shift 3
    local args=()
    local header
    for header in "$@"; do
        args+=(-H "$header")
    done

    info "发起请求（客户端将在 $max_seconds 秒内断开）：$name"
    curl -s -N --max-time "$max_seconds" \
        -X POST "$GATEWAY_URL/v1/chat/completions" \
        -H "content-type: application/json" \
        "${args[@]}" \
        -d "$body" >/dev/null 2>&1
    info "客户端已断开"
}

# 正常完成一次请求，输出响应体。
call_gateway() {
    local body="$1"
    shift
    local args=()
    local header
    for header in "$@"; do
        args+=(-H "$header")
    done

    curl -s --max-time 30 \
        -X POST "$GATEWAY_URL/v1/chat/completions" \
        -H "content-type: application/json" \
        "${args[@]}" \
        -d "$body" 2>&1
}

# 请求网关的管理接口，输出原始响应。
gateway_admin() {
    local path="$1" body="${2:-\{\}}"
    curl -s --max-time 30 \
        -X POST "$GATEWAY_URL/___/$path" \
        -H "content-type: application/json" \
        -d "$body" 2>&1
}

# ---------------------------------------------------------------------------
# 前置检查
# ---------------------------------------------------------------------------

# 确认两个服务都在运行；未运行则提示先执行 test_mock_run.sh。
require_services() {
    local missing=0
    if ! port_listening "$GATEWAY_PORT"; then
        warn "网关未在端口 $GATEWAY_PORT 运行"
        missing=1
    fi
    if ! port_listening "$MOCK_PORT"; then
        warn "mock 上游未在端口 $MOCK_PORT 运行"
        missing=1
    fi
    if [ "$missing" -ne 0 ]; then
        warn "请通过 scripts/test_mock_run.sh 统一启动服务"
        return 1
    fi
    return 0
}

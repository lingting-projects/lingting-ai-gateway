#!/usr/bin/env bash
# 取消行为验证入口：编译并启动 mock 上游与网关，按顺序运行全部 test_mock_*.sh，最后停止服务。
#
# 用法：
#     bash scripts/test_mock_run.sh
#
# 依赖 migrations/0013_mock_provider.sql 提供的 mock 供应商与 mock-chat 模型。

set -uo pipefail
source "$(dirname "$0")/test_common.sh"

FAILED_SCRIPTS=()
PASSED_SCRIPTS=()

# ---------------------------------------------------------------------------
# 服务启停
# ---------------------------------------------------------------------------

build_binaries() {
    title "编译 bin-server 与 bin-mock-ai"

    # sqlx::migrate! 是编译期宏，cargo 不追踪 migrations/ 目录变化，
    # 新增迁移后必须强制重编 lib-db，否则二进制里仍是旧的迁移集。
    touch "$ROOT_DIR/crates/lib-db/src/lib.rs"

    if ! cargo build --manifest-path "$ROOT_DIR/Cargo.toml" -p bin-server -p bin-mock-ai; then
        fail "编译失败"
        exit 1
    fi
    ok "编译完成"
}
stop_services() {
    title "停止服务"
    stop_port "$MOCK_PORT" "mock 上游"
    stop_port "$GATEWAY_PORT" "网关"
    rm -f "$PID_FILE"
    ok "服务已停止"
}

start_services() {
    title "启动服务"
    mkdir -p "$TEST_OUT_DIR"

    # 先清理残留进程，避免端口占用。
    stop_port "$MOCK_PORT" "mock 上游"
    stop_port "$GATEWAY_PORT" "网关"

    : > "$MOCK_LOG"
    : > "$GATEWAY_STDOUT"

    info "启动 mock 上游（端口 $MOCK_PORT）"
    "$ROOT_DIR/target/debug/bin-mock-ai.exe" >>"$MOCK_LOG" 2>&1 &
    echo "mock=$!" >>"$PID_FILE"
    if ! wait_for_port "$MOCK_PORT" 30; then
        fail "mock 上游启动超时"
        tail -20 "$MOCK_LOG" 2>/dev/null | sed 's/^/    /'
        exit 1
    fi
    ok "mock 上游已就绪"

    info "启动网关（端口 $GATEWAY_PORT）"
    "$ROOT_DIR/target/debug/bin-server.exe" >>"$GATEWAY_STDOUT" 2>&1 &
    echo "gateway=$!" >>"$PID_FILE"
    if ! wait_for_port "$GATEWAY_PORT" 60; then
        fail "网关启动超时"
        tail -20 "$GATEWAY_STDOUT" 2>/dev/null | sed 's/^/    /'
        exit 1
    fi
    ok "网关已就绪"
    # 给网关一点时间完成启动后的首次数据库初始化。
    sleep 2
}


# ---------------------------------------------------------------------------
# 场景执行
# ---------------------------------------------------------------------------

# 准备测试所需的全局配置：AI 接口需要允许匿名访问，否则请求在鉴权阶段就被拒绝。
prepare_config() {
    title "准备测试配置"

    local response
    response="$(gateway_admin "kv-config/upsert" \
        '{"configKey":"ALLOW_ANONYMOUS","configValue":"true"}')"
    if ! printf '%s' "$response" | grep -qF '"code":200'; then
        fail "开启匿名访问失败：$response"
        exit 1
    fi
    ok "已开启匿名访问（allow_anonymous=true）"

    # 关闭调试模式，避免日志体积影响读取。
    gateway_admin "kv-config/upsert" \
        '{"configKey":"DEBUG_MODE","configValue":"false"}' >/dev/null
    ok "已关闭调试模式（debug_mode=false）"
}

run_scenarios() {
    title "运行场景脚本"

    local script name
    for script in "$TEST_COMMON_DIR"/test_mock_*.sh; do
        name="$(basename "$script")"
        # 跳过本入口脚本与公共库。
        case "$name" in
            test_mock_run.sh | test_common.sh) continue ;;
        esac

        if bash "$script"; then
            PASSED_SCRIPTS+=("$name")
        else
            FAILED_SCRIPTS+=("$name")
        fi
    done
}

# ---------------------------------------------------------------------------
# 结果汇总
# ---------------------------------------------------------------------------

show_request_log_state() {
    title "请求日志落库状态（最近 8 条）"
    local response
    response="$(gateway_admin "request/main/page" \
        '{"current":1,"size":8,"sorts":[{"field":"id","desc":true}]}')"

    if [ -z "$response" ]; then
        warn "未取到请求日志，网关可能未就绪"
        return
    fi

    printf '%s' "$response" \
        | grep -oE '"(id|status|endTime|errorMessage)":[^,}]*' \
        | head -40 \
        | sed 's/^/    /'
}

print_summary() {
    title "总体结果"

    if [ ${#PASSED_SCRIPTS[@]} -gt 0 ]; then
        printf '%s通过%s\n' "$C_GREEN" "$C_RESET"
        local name
        for name in "${PASSED_SCRIPTS[@]}"; do
            printf '    %s\n' "$name"
        done
    fi

    if [ ${#FAILED_SCRIPTS[@]} -gt 0 ]; then
        printf '%s失败%s\n' "$C_RED" "$C_RESET"
        local name
        for name in "${FAILED_SCRIPTS[@]}"; do
            printf '    %s\n' "$name"
        done
        printf '\n%s说明：基线阶段预期出现失败项，失败即代表待修复的问题。%s\n' "$C_YELLOW" "$C_RESET"
        return 1
    fi

    printf '\n全部场景通过。\n'
    return 0
}

# ---------------------------------------------------------------------------
# 主流程
# ---------------------------------------------------------------------------

main() {
    title "取消行为验证（基线）"

    build_binaries
    start_services
    prepare_config

    run_scenarios

    show_request_log_state

    local result=0
    print_summary || result=1

    stop_services
    return "$result"
}

main

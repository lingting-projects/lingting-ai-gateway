#!/usr/bin/env bash
# 场景：非流式请求，mock 延迟 8 秒才返回响应头，客户端在 2 秒时断开。
#
# 验证目标：客户端断开后，网关是否（1）取消上游转发（2）把请求日志收尾。
# 预期（修复前基线）：上游被取消，但日志停留在 processing，永不收尾。

set -uo pipefail
source "$(dirname "$0")/test_common.sh"

SCENARIO="非流式 + 响应头延迟期间断开"
title "$SCENARIO"

if ! require_services; then
    exit 1
fi

mark_gateway_log
mark_mock_log

# 客户端 2 秒断开，mock 要 8 秒才发响应头。
disconnect_at="$(now_clock)"
call_gateway_disconnect "$SCENARIO" \
    "{\"model\":\"$TEST_MODEL\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}" \
    2 \
    "x-mock-delay-ms: 8000"

# 等待 mock 的延迟窗口过去，确认它是否感知到断开。
info "等待 10 秒，观察 mock 是否感知断开……"
sleep 10

gateway_log="$(gateway_log_since_mark)"
mock_log="$(mock_log_since_mark)"

gateway_lines="$(gateway_marked_lines "$gateway_log")"
mock_lines="$(mock_marked_lines "$mock_log")"

dump "网关日志（$MARK_GATEWAY）" "$gateway_lines"
dump "mock 日志（$MARK_MOCK）" "$mock_lines"

title "$SCENARIO 判定"

# 1. 上游是否感知到客户端断开。
assert_contains "$mock_log" "$MARK_MOCK" "上游感知到客户端断开"

# 2. 网关是否感知到自己的 handler 被 drop。
assert_contains "$gateway_log" "$MARK_GATEWAY handler-drop" "网关感知到 handler 被 drop"

# 3. 转发是否真的开始了。
assert_contains "$gateway_log" "$MARK_GATEWAY forward-start" "转发已开始"

# 4. 请求日志是否收尾 —— 这是本场景要暴露的核心问题。
if printf '%s' "$gateway_log" | grep -qF "$MARK_GATEWAY finish"; then
    ok "请求日志已收尾"
else
    fail "请求日志未收尾：无 finish 记录，request_main/request_sub 将停留在 processing"
fi

# 5. 上游断开时刻与客户端断开时刻的偏差。
mock_mark_clock="$(first_clock_of "$mock_log" "$MARK_MOCK")"
if [ -n "$mock_mark_clock" ]; then
    delay="$(clock_diff "$disconnect_at" "$mock_mark_clock")"
    detail "客户端断开 $disconnect_at，上游感知 $mock_mark_clock，偏差 ${delay}s"
fi

finish_report "$SCENARIO"

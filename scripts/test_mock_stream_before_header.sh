#!/usr/bin/env bash
# 场景：流式请求，mock 延迟 8 秒才返回响应头，客户端在 2 秒时断开。
#
# 与 test_mock_nonstream_disconnect.sh 的区别：请求体带 stream=true，但断开发生在
# 网关拿到响应头之前，此时还没有进入分片转发阶段。
#
# 预期（修复前基线）：上游被取消，但日志停留在 processing。

set -uo pipefail
source "$(dirname "$0")/test_common.sh"

SCENARIO="流式 + 响应头延迟期间断开"
title "$SCENARIO"

if ! require_services; then
    exit 1
fi

mark_gateway_log
mark_mock_log

call_gateway_disconnect "$SCENARIO" \
    "{\"model\":\"$TEST_MODEL\",\"stream\":true,\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}" \
    2 \
    "x-mock-delay-ms: 8000"

info "等待 10 秒，观察 mock 是否感知断开……"
sleep 10

gateway_log="$(gateway_log_since_mark)"
mock_log="$(mock_log_since_mark)"

dump "网关日志（$MARK_GATEWAY）" "$(gateway_marked_lines "$gateway_log")"
dump "mock 日志（$MARK_MOCK）" "$(mock_marked_lines "$mock_log")"

title "$SCENARIO 判定"

assert_contains "$mock_log" "$MARK_MOCK" "上游感知到客户端断开"
assert_contains "$gateway_log" "$MARK_GATEWAY request-cancel" "网关感知到客户端断开（request-cancel）"
assert_contains "$gateway_log" "$MARK_GATEWAY forward-start" "转发已开始"

if printf '%s' "$gateway_log" | grep -qF "$MARK_GATEWAY finish"; then
    ok "请求日志已收尾"
else
    fail "请求日志未收尾：无 finish 记录，request_main/request_sub 将停留在 processing"
fi

mock_mark_clock="$(first_clock_of "$mock_log" "$MARK_MOCK")"
if [ -n "$mock_mark_clock" ]; then
    delay="$(clock_diff "$CLIENT_DISCONNECT_AT" "$mock_mark_clock")"
    detail "客户端断开 $CLIENT_DISCONNECT_AT，上游感知 $mock_mark_clock，偏差 ${delay}s"
fi

finish_report "$SCENARIO"

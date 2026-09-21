#!/usr/bin/env bash
# 场景：流式请求，mock 每 1.5 秒吐一个分片，客户端在 2 秒时断开。
#
# 关键点：断开时刻（2.0s）落在分片之间，此时网关的分片循环正阻塞在「等下一个分片」。
# 用于验证取消是否即时 —— 若网关只在拿到下一个分片后才检查客户端状态，
# 上游要等到分片 3 到达（约 3.3s）才会感知断开。
#
# 预期（修复前基线）：上游感知断开明显滞后于客户端断开。

set -uo pipefail
source "$(dirname "$0")/test_common.sh"

SCENARIO="流式 + 分片之间断开"
title "$SCENARIO"

if ! require_services; then
    exit 1
fi

mark_gateway_log
mark_mock_log

# 首片 0.3s，之后每 1.5s 一片：断开时（2.0s）正卡在分片 2 与分片 3 之间。
disconnect_at="$(now_clock)"
call_gateway_disconnect "$SCENARIO" \
    "{\"model\":\"$TEST_MODEL\",\"stream\":true,\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}" \
    2 \
    "x-mock-first-chunk-ms: 300" \
    "x-mock-chunk-interval-ms: 1500" \
    "x-mock-chunks: 6"

info "等待 8 秒，观察上游何时感知断开……"
sleep 8

gateway_log="$(gateway_log_since_mark)"
mock_log="$(mock_log_since_mark)"

dump "网关日志（$MARK_GATEWAY）" "$(gateway_marked_lines "$gateway_log")"
dump "mock 日志（$MARK_MOCK 与分片记录）" "$(printf '%s' "$mock_log" | grep -E "$MARK_MOCK|发送分片" || true)"

title "$SCENARIO 判定"

assert_contains "$mock_log" "$MARK_MOCK" "上游感知到客户端断开"

# 流式路径下 handler 已返回响应头，不应期望它被 drop；
# 取消由分片泵感知，表现为日志收尾为 cancelled。

# 流式路径下 on_cancel 会被触发，日志应当收尾为 cancelled。
if printf '%s' "$gateway_log" | grep -qF "$MARK_GATEWAY finish"; then
    ok "请求日志已收尾"
    finish_line="$(printf '%s' "$gateway_log" | grep -F "$MARK_GATEWAY finish" | head -1 | sed 's/.*\[MOCKTEST\]/[MOCKTEST]/')"
    detail "收尾记录：$finish_line"
else
    fail "请求日志未收尾：无 finish 记录"
fi

# 取消时效：客户端断开到上游感知的偏差。
mock_mark_clock="$(first_clock_of "$mock_log" "$MARK_MOCK")"
if [ -n "$mock_mark_clock" ]; then
    delay="$(clock_diff "$disconnect_at" "$mock_mark_clock")"
    detail "客户端断开 $disconnect_at，上游感知 $mock_mark_clock，偏差 ${delay}s"
    if awk -v d="$delay" 'BEGIN { exit !(d <= 1.0) }'; then
        ok "取消即时（偏差 ${delay}s ≤ 1.0s）"
    else
        fail "取消滞后（偏差 ${delay}s > 1.0s）：网关在等下一个分片才检查客户端状态"
    fi
else
    fail "未取到上游感知断开的时刻"
fi

finish_report "$SCENARIO"

#!/usr/bin/env bash
# 场景：流式请求，mock 吐完 3 个分片后挂起（不再发数据也不结束），客户端在 2 秒时断开。
#
# 关键点：断开时上游已无更多分片可发，网关的分片循环会永远阻塞在「等下一个分片」。
# 若没有独立的取消信号或超时，上游连接将永不关闭。
#
# 预期（修复前基线）：上游永远感知不到断开，连接泄漏。

set -uo pipefail
source "$(dirname "$0")/test_common.sh"

SCENARIO="流式 + 上游挂起后断开"
title "$SCENARIO"

if ! require_services; then
    exit 1
fi

mark_gateway_log
mark_mock_log

# 分片很快发完（每片 0.2s），随后 mock 进入挂起。
disconnect_at="$(now_clock)"
call_gateway_disconnect "$SCENARIO" \
    "{\"model\":\"$TEST_MODEL\",\"stream\":true,\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}" \
    2 \
    "x-mock-chunk-interval-ms: 200" \
    "x-mock-chunks: 3" \
    "x-mock-hang: true"

info "等待 12 秒，观察上游是否感知断开（挂起场景需要更长窗口）……"
sleep 12

gateway_log="$(gateway_log_since_mark)"
mock_log="$(mock_log_since_mark)"

dump "网关日志（$MARK_GATEWAY）" "$(gateway_marked_lines "$gateway_log")"
dump "mock 日志（$MARK_MOCK 与分片记录）" "$(printf '%s' "$mock_log" | grep -E "$MARK_MOCK|发送分片|挂起" || true)"

title "$SCENARIO 判定"

# 流式路径下 handler 已返回响应头，不应期望它被 drop。

if printf '%s' "$mock_log" | grep -qF "$MARK_MOCK"; then
    ok "上游感知到客户端断开"
    mock_mark_clock="$(first_clock_of "$mock_log" "$MARK_MOCK")"
    delay="$(clock_diff "$disconnect_at" "$mock_mark_clock")"
    detail "客户端断开 $disconnect_at，上游感知 $mock_mark_clock，偏差 ${delay}s"
    if awk -v d="$delay" 'BEGIN { exit !(d <= 1.0) }'; then
        ok "取消即时（偏差 ${delay}s ≤ 1.0s）"
    else
        fail "取消滞后（偏差 ${delay}s > 1.0s）"
    fi
else
    fail "上游未感知到断开：连接仍在保持，存在连接与任务泄漏"
fi

if printf '%s' "$gateway_log" | grep -qF "$MARK_GATEWAY finish"; then
    ok "请求日志已收尾"
else
    fail "请求日志未收尾：无 finish 记录"
fi

finish_report "$SCENARIO"

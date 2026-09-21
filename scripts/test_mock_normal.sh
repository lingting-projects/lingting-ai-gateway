#!/usr/bin/env bash
# 场景：对照组 —— 流式请求正常跑完，客户端不提前断开。
#
# 用于确认 mock 与网关链路本身是通的：若本场景不通过，其它断开场景的结果没有意义。

set -uo pipefail
source "$(dirname "$0")/test_common.sh"

SCENARIO="对照组：流式正常完成"
title "$SCENARIO"

if ! require_services; then
    exit 1
fi

mark_gateway_log
mark_mock_log

info "发起请求并等待完整响应……"
response="$(call_gateway \
    "{\"model\":\"$TEST_MODEL\",\"stream\":true,\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}" \
    "x-mock-chunk-interval-ms: 100" \
    "x-mock-chunks: 3")"

sleep 3

gateway_log="$(gateway_log_since_mark)"
mock_log="$(mock_log_since_mark)"

dump "响应体（前 3 行）" "$(printf '%s' "$response" | head -3)"
dump "网关日志（$MARK_GATEWAY）" "$(gateway_marked_lines "$gateway_log")"
dump "mock 日志（分片记录）" "$(printf '%s' "$mock_log" | grep -F "发送分片" || true)"

title "$SCENARIO 判定"

assert_contains "$response" "chunk-0" "响应包含首个分片内容"
assert_contains "$response" "[DONE]" "响应包含结束标记"
assert_contains "$gateway_log" "$MARK_GATEWAY finish status=success" "日志收尾为 success"
assert_contains "$mock_log" "响应流正常结束" "上游响应流正常结束"

finish_report "$SCENARIO"

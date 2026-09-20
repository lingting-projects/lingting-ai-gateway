-- 请求日志扩展：保存请求头与返回头。
-- 仅在调试模式下写入；写入前已移除逐跳头（connection、keep-alive 等），鉴权值已脱敏。

alter table request_main
    add column if not exists request_headers  jsonb,
    add column if not exists response_headers jsonb;

alter table request_sub
    add column if not exists request_headers  jsonb,
    add column if not exists response_headers jsonb;

comment on column request_main.request_headers is '网关收到的请求头，仅调试模式记录，已移除逐跳头';
comment on column request_main.response_headers is '返回给客户端的响应头，仅调试模式记录，已移除逐跳头';
comment on column request_sub.request_headers is '实际转发给供应商的请求头，仅调试模式记录，已移除逐跳头且鉴权已脱敏';
comment on column request_sub.response_headers is '供应商返回的响应头，仅调试模式记录，已移除逐跳头';

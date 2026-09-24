-- 供应商请求 ID 由单个改为集合：响应体 id 与响应头 x-oneapi-request-id 都可能给出。

alter table request_main
    drop column if exists provider_request_id,
    add column if not exists provider_request_ids jsonb not null default '[]'::jsonb;

alter table request_sub
    drop column if exists provider_request_id,
    add column if not exists provider_request_ids jsonb not null default '[]'::jsonb;

comment on column request_main.provider_request_ids is '供应商返回的请求 ID 集合';
comment on column request_sub.provider_request_ids is '供应商返回的请求 ID 集合';

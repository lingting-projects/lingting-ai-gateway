-- lingting-ai-gateway 初始化脚本
-- 说明：所有时间字段统一使用毫秒时间戳（bigint），状态字段使用小写字符串。

-- 供应商
create table if not exists provider
(
    id           bigserial primary key,
    name         text        not null,
    display_name text        not null default '',
    base_url     text        not null,
    api_key      text        not null default '',
    priority     integer     not null default 0,
    enabled      boolean     not null default true,
    config       jsonb       not null default '{}'::jsonb,
    create_time  bigint      not null,
    update_time  bigint      not null
);

create unique index if not exists uk_provider_name on provider (name);

comment on table provider is '供应商';
comment on column provider.priority is '优先级，数值越小越优先';
comment on column provider.config is '供应商扩展配置';

-- 供应商模型
create table if not exists provider_model
(
    id              bigserial primary key,
    provider_id     bigint  not null references provider (id) on delete cascade,
    model           text    not null,
    display_name    text    not null default '',
    inference_level text    not null default 'standard',
    enabled         boolean not null default true,
    create_time     bigint  not null,
    update_time     bigint  not null
);

create unique index if not exists uk_provider_model on provider_model (provider_id, model);
create index if not exists idx_provider_model_model on provider_model (model);

comment on table provider_model is '供应商模型';
comment on column provider_model.inference_level is '推理级别：standard/economy/performance';

-- 主请求日志
create table if not exists request_main
(
    id                  bigserial primary key,
    request_id          text    not null,
    trace_id            text    not null default '',
    client_request_id   text    not null default '',
    session_id          text    not null default '',
    client_ip           text    not null default '',
    user_agent          text    not null default '',
    credential_id       text    not null default '',
    model               text    not null default '',
    stream              boolean not null default false,
    method              text    not null default '',
    path                text    not null default '',
    request_params      jsonb,
    status              text    not null default 'processing',
    current_status      text    not null default '',
    error_type          text    not null default '',
    error_code          text    not null default '',
    error_message       text    not null default '',
    provider_count      integer not null default 0,
    return_model        text    not null default '',
    input_tokens        bigint  not null default 0,
    output_tokens       bigint  not null default 0,
    cache_read_tokens   bigint  not null default 0,
    cache_write_tokens  bigint  not null default 0,
    inference_tokens    bigint  not null default 0,
    read_tokens         bigint  not null default 0,
    write_tokens        bigint  not null default 0,
    total_tokens        bigint  not null default 0,
    http_status         integer,
    finish_reason       text    not null default '',
    provider_request_id text    not null default '',
    response_content    text,
    start_time          bigint  not null,
    end_time            bigint,
    duration_ms         bigint,
    create_time         bigint  not null
);

create unique index if not exists uk_request_main_request_id on request_main (request_id);
create index if not exists idx_request_main_create_time on request_main (create_time desc);
create index if not exists idx_request_main_model on request_main (model);
create index if not exists idx_request_main_session_id on request_main (session_id);

comment on table request_main is '主请求日志';
comment on column request_main.status is '状态：processing/success/failed/cancelled';
comment on column request_main.current_status is '当前处理状态描述';

-- 子请求日志
create table if not exists request_sub
(
    id                  bigserial primary key,
    main_request_id     bigint  not null references request_main (id) on delete cascade,
    provider_id         bigint  not null,
    provider_name       text    not null default '',
    model               text    not null default '',
    provider_url        text    not null default '',
    request_params      jsonb,
    status              text    not null default 'processing',
    error_type          text    not null default '',
    error_code          text    not null default '',
    error_message       text    not null default '',
    return_model        text    not null default '',
    input_tokens        bigint  not null default 0,
    output_tokens       bigint  not null default 0,
    cache_read_tokens   bigint  not null default 0,
    cache_write_tokens  bigint  not null default 0,
    inference_tokens    bigint  not null default 0,
    read_tokens         bigint  not null default 0,
    write_tokens        bigint  not null default 0,
    total_tokens        bigint  not null default 0,
    http_status         integer,
    finish_reason       text    not null default '',
    provider_request_id text    not null default '',
    response_content    text,
    start_time          bigint  not null,
    end_time            bigint,
    duration_ms         bigint,
    create_time         bigint  not null
);

create index if not exists idx_request_sub_main_request_id on request_sub (main_request_id);
create index if not exists idx_request_sub_create_time on request_sub (create_time desc);

comment on table request_sub is '子请求日志';

-- API Key
create table if not exists api_key
(
    id          bigserial primary key,
    name        text    not null default '',
    key_hash    text    not null,
    enabled     boolean not null default true,
    deleted     boolean not null default false,
    remark      text    not null default '',
    create_time bigint  not null,
    update_time bigint  not null
);

create unique index if not exists uk_api_key_key_hash on api_key (key_hash);

comment on table api_key is 'API Key，仅保存原始 key 的 sha1 值';

-- 全局配置
create table if not exists kv_config
(
    id           bigserial primary key,
    config_key   text   not null,
    config_value text   not null default '',
    description  text   not null default '',
    create_time  bigint not null,
    update_time  bigint not null
);

create unique index if not exists uk_kv_config_key on kv_config (config_key);

comment on table kv_config is '全局配置';

-- 种子数据
insert into provider (id, name, display_name, base_url, api_key, priority, enabled, create_time, update_time)
values (1, 'openai', 'OpenAI', 'https://api.openai.com/v1', '', 0, true, 0, 0)
on conflict (id) do nothing;

insert into kv_config (id, config_key, config_value, description, create_time, update_time)
values (1, 'allow_anonymous', 'false', '是否允许匿名访问 AI 接口', 0, 0),
       (2, 'debug_mode', 'false', '是否开启调试模式，开启后记录完整请求参数与返回内容', 0, 0),
       (3, 'admin_token', '', '管理接口令牌，保存 sha1 值，为空表示不校验', 0, 0)
on conflict (id) do nothing;

select setval('provider_id_seq', (select max(id) from provider));
select setval('kv_config_id_seq', (select max(id) from kv_config));

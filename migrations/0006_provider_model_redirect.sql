-- 模型名称映射：把非官方模型名映射到官方模型名，用于复用官方模型的基础配置。

create table if not exists provider_model_redirect
(
    id          bigserial primary key,
    source      text   not null,
    target      text   not null,
    create_time bigint not null
);

create unique index if not exists uk_provider_model_redirect_source on provider_model_redirect (source);
create index if not exists idx_provider_model_redirect_target on provider_model_redirect (target);

comment on table provider_model_redirect is '模型名称映射';
comment on column provider_model_redirect.source is '非官方模型名';
comment on column provider_model_redirect.target is '映射到的官方模型名';

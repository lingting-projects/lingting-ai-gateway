-- 供应商模型参数扩展：推理能力、推理级别、上下文窗口与能力标记。
-- 移除 inference_level：推理级别改由模型自定义的字符串列表表达，见 levels 与 level_default。

alter table provider_model
    add column if not exists reasoning        boolean not null default false,
    add column if not exists levels           jsonb   not null default '[]'::jsonb,
    add column if not exists level_default    text    not null default '',
    add column if not exists context_window   bigint  not null default 0,
    add column if not exists max_tokens       bigint  not null default 0,
    add column if not exists support_tools    boolean not null default false,
    add column if not exists support_vision   boolean not null default false,
    add column if not exists support_stream   boolean not null default true,
    add column if not exists support_json     boolean not null default false,
    add column if not exists support_cache    boolean not null default false,
    add column if not exists knowledge_cutoff text    not null default '',
    add column if not exists release_date     text    not null default '';

alter table provider_model
    drop column if exists inference_level;

comment on column provider_model.reasoning is '是否推理模型，会产出思维链';
comment on column provider_model.levels is '支持的推理级别列表，级别名称由厂商与模型自行定义';
comment on column provider_model.level_default is '默认推理级别，空串表示由上游决定';
comment on column provider_model.context_window is '上下文窗口上限（输入与输出 token 合计），0 表示未知';
comment on column provider_model.max_tokens is '单次响应最大输出 token，0 表示未知';
comment on column provider_model.support_tools is '是否支持函数调用';
comment on column provider_model.support_vision is '是否支持图片输入';
comment on column provider_model.support_stream is '是否支持流式返回';
comment on column provider_model.support_json is '是否支持结构化输出';
comment on column provider_model.support_cache is '是否支持提示词缓存';
comment on column provider_model.knowledge_cutoff is '知识截止日期，如 2024-06';
comment on column provider_model.release_date is '模型发布日期，如 2024-05-13';

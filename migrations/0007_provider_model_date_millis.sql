-- 知识截止日期与模型发布日期改为毫秒级时间戳，与其余时间字段保持一致。
-- 旧值是可读文本（如 2024-06），无法无损转换，统一置 0 表示未知，由模型同步重新写入。

alter table provider_model
    alter column knowledge_cutoff drop default,
    alter column release_date     drop default;

alter table provider_model
    alter column knowledge_cutoff type bigint using 0,
    alter column release_date     type bigint using 0;

alter table provider_model
    alter column knowledge_cutoff set default 0,
    alter column release_date     set default 0;

comment on column provider_model.knowledge_cutoff is '知识截止日期（毫秒时间戳），0 表示未知';
comment on column provider_model.release_date is '模型发布日期（毫秒时间戳），0 表示未知';

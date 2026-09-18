-- 供应商逻辑删除时间，0 表示未删除，删除时写入毫秒时间戳。
alter table provider
    add column if not exists deleted_at bigint not null default 0;

comment on column provider.deleted_at is '逻辑删除时间（毫秒时间戳），0 表示未删除';

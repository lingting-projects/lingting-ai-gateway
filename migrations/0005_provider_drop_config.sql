-- 移除供应商扩展配置字段。

alter table provider
    drop column if exists config;

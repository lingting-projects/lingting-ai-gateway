-- 移除全部外键约束。
-- 业务表关联关系改由业务层维护，同时允许写入 provider_id 为负数的默认模型数据。

alter table provider_model
    drop constraint if exists provider_model_provider_id_fkey;

alter table request_sub
    drop constraint if exists request_sub_main_request_id_fkey;

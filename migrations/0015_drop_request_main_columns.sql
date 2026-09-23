-- 移除主请求日志中冗余的 request_id 与 provider_count。
--
-- request_id 由网关自行生成，与 client_request_id（客户端传入的 x-request-id）语义重复，
-- 且没有任何调用方按它查询；provider_count 记录匹配到的供应商数量，同样无实际消费方。

drop index if exists uk_request_main_request_id;

alter table request_main
    drop column if exists request_id,
    drop column if exists provider_count;

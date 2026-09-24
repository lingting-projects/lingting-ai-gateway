-- 主请求日志补充首字时间：由子请求日志完成时同步写入，两者取同一个值。
-- 非流式请求与完成时间一致；流式请求为首个分片返回时间，未产生分片时为空。

alter table request_main
    add column if not exists first_chunk_time bigint;

comment on column request_main.first_chunk_time is '首字时间（毫秒时间戳），非流式与完成时间一致，为空表示未产生分片';

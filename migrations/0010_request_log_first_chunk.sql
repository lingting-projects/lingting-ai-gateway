-- 请求日志时间补充：子请求记录首字时间。
-- 非流式请求的首字时间与完成时间一致；流式请求为首个分片返回时间，未产生分片时为空。

alter table request_sub
    add column if not exists first_chunk_time bigint;

comment on column request_main.start_time is '请求收到时间（毫秒时间戳）';
comment on column request_main.end_time is '请求完成时间（毫秒时间戳），为空表示未结束';
comment on column request_main.duration_ms is '总耗时（毫秒），即完成时间减去收到时间';
comment on column request_sub.start_time is '转发开始时间（毫秒时间戳）';
comment on column request_sub.first_chunk_time is '首字时间（毫秒时间戳），非流式与完成时间一致，为空表示未产生分片';
comment on column request_sub.end_time is '转发完成时间（毫秒时间戳），为空表示未结束';
comment on column request_sub.duration_ms is '转发耗时（毫秒），即完成时间减去开始时间';

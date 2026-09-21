-- 用 mock 上游替换 0001_init.sql 插入的 openai 供应商，供 scripts/test_mock_run.sh 做取消行为验证。
--
-- 使用前提：必须先自行启动 mock 服务
--
--     cargo run -p bin-mock-ai
--
-- mock 监听 127.0.0.1:26383，实现 /v1/models、/v1/chat/completions、/v1/responses。

delete from provider where name = 'openai';

insert into provider (name, display_name, base_url, api_key, priority, enabled, create_time, update_time)
values ('mock',
        'mock 上游（需自行启动 bin-mock-ai）',
        'http://127.0.0.1:26383/v1',
        'mock',
        0,
        true,
        0,
        0)
on conflict (name) do update
    set display_name = excluded.display_name,
        base_url     = excluded.base_url,
        api_key      = excluded.api_key,
        priority     = excluded.priority,
        enabled      = excluded.enabled,
        update_time  = excluded.update_time;

comment on column provider.display_name is '供应商展示名';

-- mock 模型：与 bin-mock-ai 的 /v1/models 返回值一致，省去手工同步。
insert into provider_model (provider_id, model, display_name, reasoning, levels, level_default,
                            context_window, max_tokens, support_tools, support_vision,
                            support_stream, support_json, support_cache,
                            knowledge_cutoff, release_date, enabled, create_time, update_time)
select p.id, 'mock-chat', 'mock 对话模型', false, '[]'::jsonb, '',
       0, 0, false, false, true, false, false, 0, 0, true, 0, 0
from provider p
where p.name = 'mock'
on conflict (provider_id, model) do nothing;

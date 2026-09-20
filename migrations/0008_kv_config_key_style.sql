-- 全局配置键统一为 `_` 分隔的全大写形式（与 KvConfigKey 的 Display / FromStr 一致）。
-- 同时写入服务运行绑定信息的默认值；端口为 0 时由操作系统分配随机端口。

update kv_config set config_key = upper(config_key) where config_key <> upper(config_key);

comment on column kv_config.config_key is '配置键，`_` 分隔的全大写形式，取值见 KvConfigKey';

insert into kv_config (config_key, config_value, description, create_time, update_time)
values ('BIND_ADDRESS', '127.0.0.1', '服务绑定地址', 0, 0),
       ('BIND_PORT', '26380', '服务绑定端口，0 表示随机端口', 0, 0)
on conflict (config_key) do nothing;

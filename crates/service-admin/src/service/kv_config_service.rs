use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::{PgPool, use_pool};
use types_admin::KvConfigKey;
use types_admin::dto::{KvConfigQO, KvConfigUpdatePO};
use types_admin::entity::KvConfig;

use crate::repository::KvConfigRepository;

/// 未配置或配置非法时使用的默认绑定地址。
const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";

/// 未配置或配置非法时使用的默认绑定端口。
const DEFAULT_BIND_PORT: i32 = 26380;

/// 服务运行绑定信息。
#[derive(Debug, Clone)]
pub struct ServerBind {
    /// 绑定地址。
    pub address: String,
    /// 绑定端口，0 表示随机端口。
    pub port: i32,
}

/// 全局配置业务逻辑。
pub struct KvConfigService {
    repository: KvConfigRepository,
}

impl KvConfigService {
    /// 绑定当前请求的数据库连接池。
    pub fn new() -> Result<Self> {
        Ok(Self::from(use_pool()?))
    }

    /// 绑定指定连接池，供无请求上下文的场景（如服务启动）使用。
    pub fn from(pool: PgPool) -> Self {
        Self {
            repository: KvConfigRepository::new(pool),
        }
    }

    /// 分页查询全局配置。
    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &KvConfigQO,
    ) -> Result<PaginationResult<KvConfig>> {
        self.repository.page(pagination, conditions).await
    }

    /// 查询全部全局配置。
    pub async fn find_all(&self) -> Result<Vec<KvConfig>> {
        self.repository.find_all().await
    }

    /// 按配置键批量查询配置，避免逐个 key 查询。
    pub async fn find_keys(&self, keys: &[KvConfigKey]) -> Result<Vec<KvConfig>> {
        let keys = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
        let keys = keys.iter().map(String::as_str).collect::<Vec<_>>();
        self.repository.find_keys(&keys).await
    }

    /// 读取单个配置值。
    pub async fn find_value(&self, config_key: KvConfigKey) -> Result<Option<String>> {
        self.repository.find_value(&config_key.to_string()).await
    }

    /// 服务运行绑定信息：读取绑定地址与端口，未配置或无法解析时使用默认值。
    pub async fn server_bind(&self) -> Result<ServerBind> {
        let configs = self
            .find_keys(&[KvConfigKey::BindAddress, KvConfigKey::BindPort])
            .await?;

        let mut bind = ServerBind {
            address: DEFAULT_BIND_ADDRESS.to_string(),
            port: DEFAULT_BIND_PORT,
        };
        for config in configs {
            match config.config_key.parse::<KvConfigKey>() {
                Ok(KvConfigKey::BindAddress) => {
                    let address = config.config_value.trim();
                    if !address.is_empty() {
                        bind.address = address.to_string();
                    }
                }
                Ok(KvConfigKey::BindPort) => {
                    if let Ok(port) = config.config_value.trim().parse::<i32>() {
                        bind.port = port;
                    }
                }
                _ => {}
            }
        }

        Ok(bind)
    }

    /// 更新配置值。
    pub async fn update(&self, params: &KvConfigUpdatePO) -> Result<()> {
        self.repository
            .update_value(&params.config_key, &params.config_value)
            .await
    }

    /// 写入或更新配置：key 存在则更新，不存在则新增。
    pub async fn upsert(&self, params: &KvConfigUpdatePO) -> Result<()> {
        self.repository
            .upsert(&params.config_key, &params.config_value)
            .await
    }
}

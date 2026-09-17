## lingting-ai-gateway 实现

- 本次改动为重写, 只是仓库复用, 不用检索和依赖git历史记录
- 本次仅支持openai

1. AI 请求进入服务, 仅支持 openai 格式请求进入: chat, models 接口
2. 根据配置鉴权:
    1. 禁止匿名访问, 校验 Authorization 值是否有效
    2. 允许匿名访问, 忽略 Authorization 值
3. 写入主请求日志, 记录:
    - 来源信息: IP、User-Agent、凭证标识、请求 ID
    - 请求信息: 请求地址、请求方法、请求模型、Stream、请求时间
    - 请求参数: 完整请求参数 (仅调试模式开启时记录)
    - 状态: 进行中、当前处理状态
4. 获取拥有请求模型的供应商, 没有返回异常
    - 有供应商: 更新主请求日志, 记录当前处理状态、匹配到的供应商数量
    - 没有供应商: 更新主请求日志, 记录异常信息、状态 (失败), 返回异常
5. 供应商按优先级升序排序, 优先级一致则新增加的供应商排后面
6. 根据配置及请求参数初始化客户端:
    - stream=false, 初始化普通客户端
    - stream=true, 初始化流式客户端
7. 客户端发起请求, 写入子请求日志, 记录:
    - 主请求 ID
    - 供应商
    - 请求信息: 请求模型、请求地址、请求参数摘要
    - 请求时间
    - 状态: 进行中
8. 客户端请求成功: 更新子请求日志, 记录:
    - 返回模型
    - Token 计数:
        - 输入 Token
        - 输出 Token
        - 缓存读取 Token
        - 缓存写入 Token
        - 推理 Token
        - 读取 Token
        - 写入 Token
        - 总 Token
    - 请求信息
    - 返回信息: HTTP 状态、finish_reason、供应商请求 ID 等
    - 返回内容 (仅调试模式开启时)
    - 状态: 完成
    - 完成时间、耗时
9. 客户端请求成功: 更新主请求日志, 记录:
    - 返回模型
    - Token 计数:
        - 输入 Token
        - 输出 Token
        - 缓存读取 Token
        - 缓存写入 Token
        - 推理 Token
        - 读取 Token
        - 写入 Token
        - 总 Token
    - 请求信息
    - 返回信息: HTTP 状态、finish_reason、供应商请求 ID 等
    - 返回内容 (仅调试模式开启时)
    - 状态: 完成
    - 完成时间、耗时
10. 客户端请求失败: 更新子请求日志, 记录:
    - 请求信息
    - 异常信息: 异常类型、异常编码、异常消息
    - 返回内容
    - Token 计数 (如果供应商已返回或已产生)
    - 状态: 失败
    - 完成时间、耗时
11. 客户端请求失败: 更新主请求日志, 记录:
    - 请求信息
    - 异常信息: 异常类型、异常编码、异常消息
    - 返回内容
    - Token 计数 (如果供应商已返回或已产生)
    - 状态: 失败
    - 完成时间、耗时

### openai客户端

1. 所有客户端通用方法放在 lib-provider中
2. openai 客户端实现在 lib-provider-openai 中
3. 客户端不允许操作请求日志, 提供开始回调,成功回调和失败回调, 回调中处理

#### 客户端

- 根据请求参数 stream 初始化不同的客户端
  - stream=false: 初始化普通客户端
  - stream=true: 初始化流式客户端
- 普通客户端使用 stream=false 向供应商发起请求，等待完整响应后返回
- 流式客户端使用 stream=true 向供应商发起请求，按分块持续返回

1. 普通客户端
   - 发起 stream=false 请求
   - 等待供应商返回完整响应
   - 获取完整返回内容、返回模型、Token 计数、返回信息
   - 请求完成后调用成功/失败回调
   - 成功/失败回调负责更新子请求日志、主请求日志
2. 流式客户端
   - 发起 stream=true 请求
   - 供应商每返回一个分块, 通过异步回调写入返回值
   - 持续分块内容给客户端
   - 持续累计已返回内容及已获取的 Token 计数
   - 请求正常结束后调用成功回调
   - 请求中断、超时、客户端取消、解析异常或供应商返回错误后调用失败回调
   - 请求失败时, 将当前已经累计的 Token 计数、已返回内容、返回模型、返回信息、异常信息等一并写入子请求日志和主请求日志
   - 即使流式请求最终失败, 已产生的 Token 计数及已返回内容也不能丢失
3. 客户端等待请求完成后处理请求日志
   - 普通客户端: 收到完整响应后处理
   - 流式客户端: 收到流结束后处理
   - 流式请求异常结束: 使用异常发生前已累计的数据处理请求日志
   - 日志处理通过成功/失败回调执行，不影响正常响应分块

### openai 接口

- 放在 lib-web/src/openai.rs 下

1. agent使用接口: 按照主流程走
2. 可用模型查询接口: 供应商模型表 查询名称, 去重, 筛掉禁用的模型然后返回

### 请求日志

1. 要记录详细的源信息, 源请求信息, 要包含客户端的 各种请求id, 会话id 等等, 统一到 clientRequestId, sessionId, providerRequestId, traceId 等字段中, 便于排查追踪
2. 仅记录 ai 的请求信息, 不记录管理接口的请求
3. 状态由 进行中, 成功, 失败, 取消 组成; 取消表示客户端取消请求, 要进行取消客户端操作

### 供应商处理服务

- service-provider

1. 提供 ProviderFactory, 传入模型返回 ProvicerServiceClient
2. 提供供应商模型更新任务

#### 提供供应商模型更新任务

- 传入供应商, 构造客户端拉取模型, 在一个事务中替换模型, 删除已经没有的, 新增不存在的

1. 异步函数, 内部起一个线程自己去跑,不需要等待完成
2. 程序启动时为每个供应商跑一个这个任务, 对外提供方法, 由 bin-server 启动时调用, 异步进行, 不阻塞启动
3. 内部要利用 pg的锁, 避免同时对一个供应商进行更新导致锁竞争, 拉取最新模型前就要上锁, 避免无效请求,有个供应商会有请求频率限制

#### ProvicerServiceClient

- 提供 start 异步无参数方法
- 回调异常不能影响接口正常完成
- 要尝试从请求头和请求参数中获取 会话id(session_id)用于 用于关联一个会话的所有请求, 用于支持会话的token统计
- 该客户端为非熔断客户端, 仅使用获取到的第一个供应商进行请求, 失败则直接失败

1. 根据模型,参数,供应商构造客户端
2. 构造主请求日志
3. 构造子请求日志
4. 发起请求, 注入成功,失败回调,取消回调, 返回值转发器
5. 等待回调信号

#### 返回值转发器

1. 两种类型, 普通的和流式的
2. 记录每次转发的 token信息, 返回内容(仅调试模式), 时间
3. 回调中根据转发器内部数据设置请求日志相关数据
4. token计费信息要明确, 输入,输出, 读取(包含缓存), 写入, 缓存读取 等具体的token信息, 没有使用 0

### 类型定义

- types-admin 

1. 主请求日志: RequestMain
2. 子请求日志: RequestSub
3. 供应商: Provider
4. 供应商模型: ProviderModel, 多对一, 一个供应商对应多个供应商模型数据, 要支持推理级别
5. 全局配置: KvConfig
6. API Keys: ApiKey, 每条记录一个key, 用于在关闭匿名访问时, 客户端使用, 数据库不保存原始key, 仅记录key的sha1值

### 管理接口

- lib-web, 接口使用 `framework_web::web_api*` 宏标注
- 接口统一用post请求, 避免后面扩展参数维度过多时查询url过长
- 统一使用 `___` 前缀, 和 ai 使用的接口区分, 避免同名映射
- 返回值统一 R , 分页使用 Pagination* 等 framework*中的已有代码
- 参考: D:\code\projects\relayx-codes\relayx-codes-rust\lib-web\src

1. src/request.rs: 主/子 请求日志独立的分页接口; 扩展主分页接口(这个接口返回的主额外添加字段 children: RequestSub[], 查两次, 一次查main,然后汇总main.id统一查询所有sub, 再组装)
2. src/api_key.rs: APIKey管理, 删除为逻辑删除
3. src/config.rs: 全局配置管理
4. src/provider.rs: 供应商管理, 供应商模型 查看, 启用/禁用; 供应商更新保存成功后异步触发供应商模型更新任务

### 鉴权管理

- 统一在lib-web中处理
- 鉴权类型放在 lib-web-core 中
- 全局配置支持指定管理token, sha1后存储sha1值

#### 类型

1. Authorization, 字段 type: API | ADMIN | ANONYMOUS; value: String, 没有时使用空字符串
2. 获取 Authorization 请求头值
3. 将值进行sha1, 看看是否匹配某个 ApiKey, 匹配则设置为 API类型, 值为 id
4. 将值进行sha1, 看看是否匹配 管理token, 匹配则设置为 ADMIN 类型, 值为 sha1值, 管理token未指定也设置为 ADMIN类型, 值为空字符串 
5. 都不匹配设置为 ANONYMOUS, 值为 空字符串

#### openai接口鉴权

1. 允许匿名访问时, Authorization 不校验
2. 不允许匿名访问时, 上下文 Authorization 必须为 API类型且值正常
3. 匿名访问时, 请求日志中的 凭证标识 为空字符串, 非匿名访问时 凭证标识 为 Authorization.value

#### 管理接口鉴权

1. Authorization 必须为 ADMIN 类型

### lib-web 

- 在这里进行路由分发, 统一鉴权, 上下文注入
- 参考: D:\code\projects\relayx-codes\relayx-codes-rust\lib-web\src\router.rs

### bin-server

- 参考: D:\code\projects\relayx-codes\relayx-codes-rust\bin-axum\src\main.rs
- 这里进行服务启动, 日志设置, 数据库初始化

1. 数据库放在 ApplicationDirectory.data 下

### lib-db

1. 合并sql文件放在 /migrations 下. 
2. 初始化时由sqlx执行 migrations

### service-admin

- service-admin

1. src/service: 文件夹存放 表名_service.rs 用于该表业务
2. src/repository: 文件存放 表名_repository.rs 用于该表原子化操作,不负责业务
3. src/manager: 文件夹存放 领域/动作/行为.rs 用于统一某一个范围的跨表操作

### 上下文

- lib-web要统一使用的上下文, 避免无限透传

1. lib-db 提供 DbContext 和 useDb 用于内部获取数据库实例
2. framework_web::context::WebContext 要使用, 如果不满足内部要求, 允许修改 framework项目相关代码
3. lib-web-core 提供 AppContext 和 useApp, 用于内部获取部分全局配置, 如 是否开启了调试模式, 是否允许匿名访问 等
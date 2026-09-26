# lingting-ai-gateway

个人使用的轻量 AI 路由网关。持久化结构与分层说明见 [workerspace.md](workerspace.md)。

## 数据访问

### SQL 参数必须绑定

- 禁止用 `format!` 或字符串拼接把参数值写进 SQL，必须用 `bind`（`sqlx::query`）或 `push_bind`（`QueryBuilder`）。
- 原因：拼接值会绕过参数绑定，值中出现 `{`、`'` 等字符时直接变成 SQL 语法错误，同时带来注入风险。项目曾因
  `WHERE provider_id <> {DEFAULT_PROVIDER_ID}` 漏写 `format!`，导致同步时报 `syntax error at or near "{"`。
- SQL 标识符（表名、列名）无法绑定，允许拼接，但只能是代码内的常量（如各 repository 的 `COLUMNS`），不得来自外部输入。
- 数组参数同理：调用 SQL 前必须判空熔断，不得把空数组交给 `= ANY($n)`、`<> ALL($n)` 之类的表达式。

### bin-release

- 该crate 为我用来实现发布的代码

*禁止读取和修改*: 禁止读取和修改该crate下代码. 即便是搜索命令找到了位于该crate下的代码或文件, 也要忽略
*被该crate阻止进行操作*: 在进行操作时由于该crate失败, 输出操作指令和错误信息, 停止当前工作, 让我来修复后在继续
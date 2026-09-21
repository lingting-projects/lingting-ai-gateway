-- 移除恒为 0 的冗余 token 列。
--
-- read_tokens / write_tokens 与 input_tokens / output_tokens 语义重复，且没有任何上游写入点，
-- 始终为 0；[OI] 用量只上报 input、output、cached_tokens 与 reasoning_tokens。

alter table request_main
    drop column if exists read_tokens,
    drop column if exists write_tokens;

alter table request_sub
    drop column if exists read_tokens,
    drop column if exists write_tokens;

BEGIN;
-- ============================================================
-- 写入模型
--
-- provider_id = -20260918
--
-- 排除：
--   codex-auto-review  -- Codex 内部 auto-review 名称，不作为普通 API model
--   glm-5-turbo         -- 未确认官方模型 ID
-- ============================================================

INSERT INTO provider_model
(
    provider_id,
    model,
    display_name,
    enabled,
    create_time,
    update_time,

    reasoning,
    levels,
    level_default,

    context_window,
    max_tokens,

    support_tools,
    support_vision,
    support_stream,
    support_json,
    support_cache,

    knowledge_cutoff,
    release_date
)
VALUES

-- ============================================================
-- MiniMax
-- ============================================================

(
    -20260918,
    'MiniMax-M3',
    'MiniMax M3',
    true,
    0,
    0,

    true,
    '["low","medium","high"]'::jsonb,
    'medium',

    1000000,
    131072,

    true,
    true,
    true,
    true,
    true,

    0,
    0
),


-- ============================================================
-- DeepSeek
-- ============================================================

(
    -20260918,
    'deepseek-v4-flash',
    'DeepSeek V4 Flash',
    true,
    0,
    0,

    true,
    '["low","medium","high"]'::jsonb,
    'medium',

    1000000,
    0,

    true,
    true,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'deepseek-v4-flash-0731',
    'DeepSeek V4 Flash 0731',
    true,
    0,
    0,

    true,
    '["low","medium","high"]'::jsonb,
    'medium',

    1000000,
    0,

    true,
    true,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'deepseek-v4-flash-vision-exp',
    'DeepSeek V4 Flash Vision Exp',
    true,
    0,
    0,

    true,
    '["low","medium","high"]'::jsonb,
    'medium',

    1000000,
    0,

    true,
    true,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'deepseek-v4-pro',
    'DeepSeek V4 Pro',
    true,
    0,
    0,

    true,
    '["low","medium","high"]'::jsonb,
    'medium',

    1000000,
    0,

    true,
    true,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'deepseek-v4-pro-0813',
    'DeepSeek V4 Pro 0813',
    true,
    0,
    0,

    true,
    '["low","medium","high"]'::jsonb,
    'medium',

    1000000,
    0,

    true,
    true,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'deepseek-v4.1-flash',
    'DeepSeek V4.1 Flash',
    true,
    0,
    0,

    true,
    '["low","medium","high"]'::jsonb,
    'medium',

    1000000,
    0,

    true,
    true,
    true,
    true,
    true,

    0,
    0
),


-- ============================================================
-- GLM / Z.ai
-- ============================================================

(
    -20260918,
    'glm-5.2',
    'GLM-5.2',
    true,
    0,
    0,

    true,
    '["low","medium","high","max"]'::jsonb,
    'medium',

    1000000,
    131072,

    true,
    false,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'glm-5.3',
    'GLM-5.3',
    true,
    0,
    0,

    true,
    '["low","medium","high","max"]'::jsonb,
    'medium',

    1000000,
    131072,

    true,
    false,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'glm-5.3-flash',
    'GLM-5.3 Flash',
    true,
    0,
    0,

    true,
    '["low","medium","high","max"]'::jsonb,
    'medium',

    1000000,
    131072,

    true,
    true,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'glm-5',
    'GLM-5',
    true,
    0,
    0,

    true,
    '["low","medium","high","max"]'::jsonb,
    'medium',

    1000000,
    131072,

    true,
    false,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'glm-5.1',
    'GLM-5.1',
    true,
    0,
    0,

    true,
    '["low","medium","high","max"]'::jsonb,
    'medium',

    1000000,
    131072,

    true,
    false,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'glm-4.7',
    'GLM-4.7',
    true,
    0,
    0,

    true,
    '["low","medium","high","max"]'::jsonb,
    'medium',

    204800,
    131072,

    true,
    false,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'glm-4.6',
    'GLM-4.6',
    true,
    0,
    0,

    true,
    '["low","medium","high"]'::jsonb,
    'medium',

    200000,
    131072,

    true,
    false,
    true,
    false,
    true,

    0,
    0
),

(
    -20260918,
    'glm-4.5',
    'GLM-4.5',
    true,
    0,
    0,

    true,
    '["low","medium","high"]'::jsonb,
    'medium',

    128000,
    98304,

    true,
    false,
    true,
    false,
    true,

    0,
    0
),

(
    -20260918,
    'glm-4.5-air',
    'GLM-4.5 Air',
    true,
    0,
    0,

    true,
    '["low","medium","high"]'::jsonb,
    'medium',

    128000,
    98304,

    true,
    false,
    true,
    false,
    true,

    0,
    0
),


-- ============================================================
-- Kimi
-- ============================================================

(
    -20260918,
    'kimi-k2.6',
    'Kimi K2.6',
    true,
    0,
    0,

    true,
    '["low","medium","high","max"]'::jsonb,
    'medium',

    1000000,
    0,

    true,
    true,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'kimi-k2.7-code',
    'Kimi K2.7 Code',
    true,
    0,
    0,

    true,
    '["low","high","max"]'::jsonb,
    'high',

    1000000,
    0,

    true,
    false,
    true,
    true,
    true,

    0,
    0
),

(
    -20260918,
    'kimi-k3',
    'Kimi K3',
    true,
    0,
    0,

    true,
    '["low","high","max"]'::jsonb,
    'high',

    1000000,
    0,

    true,
    true,
    true,
    true,
    true,

    0,
    0
),


-- ============================================================
-- Qwen
-- ============================================================

(
    -20260918,
    'qwen3.8-max',
    'Qwen3.8 Max',
    true,
    0,
    0,

    true,
    '["low","medium","high","max"]'::jsonb,
    'medium',

    991808,
    131072,

    true,
    true,
    true,
    true,
    true,

    0,
    0
),


-- ============================================================
-- OpenAI
-- ============================================================

(
    -20260918,
    'gpt-5.3-codex-spark',
    'GPT-5.3 Codex Spark',
    true,
    0,
    0,

    true,
    '["low","medium","high"]'::jsonb,
    'medium',

    128000,
    0,

    true,
    false,
    true,
    false,
    true,

    0,
    0
),

(
    -20260918,
    'gpt-5.6-sol',
    'GPT-5.6 Sol',
    true,
    0,
    0,

    true,
    '["none","low","medium","high","xhigh","max"]'::jsonb,
    'medium',

    1050000,
    128000,

    true,
    false,
    true,
    true,
    true,

    1771200000000,
    1781913600000
),

(
    -20260918,
    'gpt-5.6-terra',
    'GPT-5.6 Terra',
    true,
    0,
    0,

    true,
    '["none","low","medium","high","xhigh","max"]'::jsonb,
    'medium',

    1050000,
    128000,

    true,
    false,
    true,
    true,
    true,

    1771200000000,
    1781913600000
),

(
    -20260918,
    'gpt-6-astra',
    'GPT-6 Astra',
    true,
    0,
    0,

    true,
    '["low","medium","high","xhigh","max"]'::jsonb,
    'high',

    1050000,
    128000,

    true,
    true,
    true,
    true,
    true,

    1777507200000,
    1777507200000
)


    ON CONFLICT (provider_id, model)
DO UPDATE SET
    display_name     = EXCLUDED.display_name,
           enabled          = EXCLUDED.enabled,
           update_time      = EXCLUDED.update_time,

           reasoning        = EXCLUDED.reasoning,
           levels           = EXCLUDED.levels,
           level_default    = EXCLUDED.level_default,

           context_window   = EXCLUDED.context_window,
           max_tokens       = EXCLUDED.max_tokens,

           support_tools    = EXCLUDED.support_tools,
           support_vision   = EXCLUDED.support_vision,
           support_stream   = EXCLUDED.support_stream,
           support_json     = EXCLUDED.support_json,
           support_cache    = EXCLUDED.support_cache,

           knowledge_cutoff = EXCLUDED.knowledge_cutoff,
           release_date     = EXCLUDED.release_date;


COMMIT;
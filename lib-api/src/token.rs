use lib_core::TokenUsage;
use serde_json::Value;

pub fn parse_usage(value: Option<&Value>) -> TokenUsage {
    let get = |path: &[&str]| {
        path.iter()
            .try_fold(value?, |current, key| current.get(*key))
            .and_then(Value::as_i64)
    };
    TokenUsage {
        input_tokens: get(&["prompt_tokens"]),
        output_tokens: get(&["completion_tokens"]),
        total_tokens: get(&["total_tokens"]),
        cache_read_input_tokens: get(&["prompt_tokens_details", "cached_tokens"]),
        cache_write_input_tokens: get(&["prompt_tokens_details", "cache_write_tokens"]),
        reasoning_tokens: get(&["completion_tokens_details", "reasoning_tokens"]),
        input_audio_tokens: get(&["prompt_tokens_details", "audio_tokens"]),
        output_audio_tokens: get(&["completion_tokens_details", "audio_tokens"]),
    }
}

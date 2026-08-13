CREATE TABLE requests (
    id BIGINT PRIMARY KEY,
    api_key_id BIGINT NOT NULL DEFAULT 0,
    client_model TEXT NOT NULL,
    provider_id BIGINT NOT NULL REFERENCES providers(id),
    provider_model TEXT,
    response_model TEXT,
    request_type TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at BIGINT NOT NULL,
    completed_at BIGINT,
    status_code INTEGER,
    input_tokens BIGINT,
    output_tokens BIGINT,
    total_tokens BIGINT,
    cache_read_input_tokens BIGINT,
    cache_write_input_tokens BIGINT,
    reasoning_tokens BIGINT,
    input_audio_tokens BIGINT,
    output_audio_tokens BIGINT,
    latency_ms BIGINT,
    error_type TEXT,
    error_code TEXT,
    error_message TEXT,
    created_at BIGINT NOT NULL
);
CREATE INDEX idx_requests_api_key_id ON requests(api_key_id);
CREATE INDEX idx_requests_provider_id ON requests(provider_id);
CREATE INDEX idx_requests_started_at ON requests(started_at);
CREATE INDEX idx_requests_status ON requests(status);

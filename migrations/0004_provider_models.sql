CREATE TABLE provider_models (
    id BIGINT PRIMARY KEY,
    provider_id BIGINT NOT NULL REFERENCES providers(id),
    model_name TEXT NOT NULL,
    upstream_model TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE(provider_id, model_name)
);
CREATE INDEX idx_provider_models_provider_id ON provider_models(provider_id);

CREATE TABLE providers (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    base_url TEXT NOT NULL,
    api_key TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    deleted_at BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
CREATE INDEX idx_providers_deleted_at ON providers(deleted_at);

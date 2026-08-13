CREATE TABLE api_keys (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    hash TEXT NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

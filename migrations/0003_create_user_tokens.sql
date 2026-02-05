-- 0003_create_user_tokens.sql
CREATE TABLE IF NOT EXISTS user_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    identity_id UUID NOT NULL REFERENCES user_identities(id) ON DELETE CASCADE,
    encrypted_access_token BYTEA NOT NULL,
    encrypted_refresh_token BYTEA,
    nonce BYTEA NOT NULL,
    nonce_refresh BYTEA,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);

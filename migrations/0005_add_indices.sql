-- 0005_add_indices.sql
CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);
CREATE INDEX IF NOT EXISTS idx_user_identities_slack_id ON user_identities (slack_id);
CREATE INDEX IF NOT EXISTS idx_magic_links_token_hash ON magic_links (token_hash);

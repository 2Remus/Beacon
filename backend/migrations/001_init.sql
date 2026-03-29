-- Create the users table
CREATE TABLE IF NOT EXISTS users (
                                     id TEXT PRIMARY KEY,        -- Keycloak UUID
                                     username TEXT NOT NULL,
                                     join_date BIGINT NOT NULL,
                                     servers TEXT[] NOT NULL DEFAULT '{}'
);


DROP TABLE IF EXISTS servers;

-- Create table with native UUID generation (Postgres 13+)
CREATE TABLE servers (
                         id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                         name TEXT NOT NULL UNIQUE,
                         container_id TEXT NOT NULL,
                         version TEXT NOT NULL,
                         status TEXT NOT NULL,
                         server_type TEXT NOT NULL,
                         created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

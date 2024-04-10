CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS todos (
    id UUID UNIQUE PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT,
    description TEXT,
    completed BOOLEAN,
    slack_user VARCHAR(24)
);
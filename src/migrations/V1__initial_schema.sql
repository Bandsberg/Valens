-- V1: Initial schema
-- UUIDs stored as hyphenated TEXT (e.g. "550e8400-e29b-41d4-a716-446655440000")

CREATE TABLE IF NOT EXISTS products (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    notes       TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS features (
    id                  TEXT PRIMARY KEY NOT NULL,
    name                TEXT NOT NULL DEFAULT '',
    description         TEXT NOT NULL DEFAULT '',
    status              TEXT NOT NULL DEFAULT '',
    notes               TEXT NOT NULL DEFAULT '',
    user_story          TEXT NOT NULL DEFAULT '',
    acceptance_criteria TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS pain_reliefs (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    notes       TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS gain_creators (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    notes       TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS customer_segments (
    id              TEXT PRIMARY KEY NOT NULL,
    name            TEXT NOT NULL DEFAULT '',
    description     TEXT NOT NULL DEFAULT '',
    notes           TEXT NOT NULL DEFAULT '',
    characteristics TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS jobs (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    notes       TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS pains (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    notes       TEXT NOT NULL DEFAULT '',
    importance  REAL NOT NULL DEFAULT 0.5
);

CREATE TABLE IF NOT EXISTS gains (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    notes       TEXT NOT NULL DEFAULT '',
    importance  REAL NOT NULL DEFAULT 0.5
);

-- Link tables (composite PK prevents duplicate links)

CREATE TABLE IF NOT EXISTS product_feature_links (
    product_id TEXT NOT NULL,
    feature_id TEXT NOT NULL,
    PRIMARY KEY (product_id, feature_id)
);

CREATE TABLE IF NOT EXISTS feature_pain_relief_links (
    feature_id     TEXT NOT NULL,
    pain_relief_id TEXT NOT NULL,
    PRIMARY KEY (feature_id, pain_relief_id)
);

CREATE TABLE IF NOT EXISTS feature_gain_creator_links (
    feature_id      TEXT NOT NULL,
    gain_creator_id TEXT NOT NULL,
    PRIMARY KEY (feature_id, gain_creator_id)
);

CREATE TABLE IF NOT EXISTS segment_job_links (
    job_id     TEXT NOT NULL,
    segment_id TEXT NOT NULL,
    PRIMARY KEY (job_id, segment_id)
);

CREATE TABLE IF NOT EXISTS job_pain_links (
    pain_id TEXT NOT NULL,
    job_id  TEXT NOT NULL,
    PRIMARY KEY (pain_id, job_id)
);

CREATE TABLE IF NOT EXISTS job_gain_links (
    gain_id TEXT NOT NULL,
    job_id  TEXT NOT NULL,
    PRIMARY KEY (gain_id, job_id)
);

-- Annotated links
-- value_type stored as TEXT: 'TableStake' | 'Differentiator'

CREATE TABLE IF NOT EXISTS pain_relief_annotations (
    pain_or_gain_id        TEXT NOT NULL,
    reliever_or_creator_id TEXT NOT NULL,
    value_type             TEXT NOT NULL DEFAULT 'Differentiator',
    strength               REAL NOT NULL DEFAULT 0.5,
    PRIMARY KEY (pain_or_gain_id, reliever_or_creator_id)
);

CREATE TABLE IF NOT EXISTS gain_creator_annotations (
    pain_or_gain_id        TEXT NOT NULL,
    reliever_or_creator_id TEXT NOT NULL,
    value_type             TEXT NOT NULL DEFAULT 'Differentiator',
    strength               REAL NOT NULL DEFAULT 0.5,
    PRIMARY KEY (pain_or_gain_id, reliever_or_creator_id)
);

DO $$
BEGIN
    CREATE TYPE exercise_kind AS ENUM ('weighted', 'bodyweight');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END
$$;

DO $$
BEGIN
    CREATE TYPE exercise_source AS ENUM ('built_in', 'user_defined');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END
$$;

DO $$
BEGIN
    CREATE TYPE muscle_group AS ENUM ('chest', 'back', 'shoulders', 'arms', 'legs', 'core');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END
$$;

DO $$
BEGIN
    CREATE TYPE weight_unit AS ENUM ('kg', 'lbs');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END
$$;

DO $$
BEGIN
    CREATE TYPE height_unit AS ENUM ('cm', 'in');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END
$$;

DO $$
BEGIN
    CREATE TYPE load_type AS ENUM ('weighted', 'bodyweight');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END
$$;

DO $$
BEGIN
    CREATE TYPE workout_source AS ENUM ('manual', 'ai_generated');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END
$$;

CREATE TABLE IF NOT EXISTS exercises (
    id UUID NOT NULL,
    user_id BIGINT NOT NULL,
    name TEXT NOT NULL,
    kind exercise_kind NOT NULL,
    muscle_group muscle_group NOT NULL,
    secondary_muscle_groups muscle_group[],
    source exercise_source NOT NULL,
    PRIMARY KEY (id, user_id)
);

CREATE INDEX IF NOT EXISTS exercises_user_id_name_idx
    ON exercises (user_id, name);

CREATE TABLE IF NOT EXISTS workouts (
    id UUID NOT NULL,
    user_id BIGINT NOT NULL,
    name TEXT,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ,
    source workout_source NOT NULL DEFAULT 'manual',
    PRIMARY KEY (id, user_id)
);

CREATE INDEX IF NOT EXISTS workouts_user_id_start_date_idx
    ON workouts (user_id, start_date DESC);

ALTER TABLE workouts
    ADD COLUMN IF NOT EXISTS source workout_source NOT NULL DEFAULT 'manual';

CREATE TABLE IF NOT EXISTS workout_exercises (
    workout_id UUID NOT NULL,
    user_id BIGINT NOT NULL,
    exercise_id UUID NOT NULL,
    entry_order INTEGER NOT NULL,
    notes TEXT,
    PRIMARY KEY (workout_id, user_id, exercise_id),
    FOREIGN KEY (workout_id, user_id)
        REFERENCES workouts (id, user_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS workout_exercises_lookup_idx
    ON workout_exercises (user_id, workout_id, entry_order);

CREATE TABLE IF NOT EXISTS performed_sets (
    workout_id UUID NOT NULL,
    user_id BIGINT NOT NULL,
    exercise_id UUID NOT NULL,
    set_order INTEGER NOT NULL,
    reps INTEGER NOT NULL CHECK (reps >= 0),
    load_type load_type NOT NULL,
    weight_value DOUBLE PRECISION,
    weight_units weight_unit,
    PRIMARY KEY (workout_id, user_id, exercise_id, set_order),
    FOREIGN KEY (workout_id, user_id, exercise_id)
        REFERENCES workout_exercises (workout_id, user_id, exercise_id)
        ON DELETE CASCADE,
    CHECK (
        (load_type = 'weighted' AND weight_value IS NOT NULL AND weight_units IS NOT NULL)
        OR
        (load_type = 'bodyweight' AND weight_value IS NULL AND weight_units IS NULL)
    )
);

CREATE INDEX IF NOT EXISTS performed_sets_lookup_idx
    ON performed_sets (user_id, workout_id, exercise_id, set_order);

CREATE TABLE IF NOT EXISTS health_params (
    user_id BIGINT PRIMARY KEY,
    weight_value DOUBLE PRECISION NOT NULL,
    weight_units weight_unit NOT NULL,
    height_value DOUBLE PRECISION NOT NULL,
    height_units height_unit NOT NULL,
    age INTEGER NOT NULL CHECK (age >= 0)
);

CREATE TABLE IF NOT EXISTS generation_jobs (
    id UUID PRIMARY KEY,
    user_id BIGINT NOT NULL,
    date DATE NOT NULL,
    status TEXT NOT NULL,
    request_fingerprint TEXT NOT NULL DEFAULT '',
    request_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    workout_id UUID NULL,
    error TEXT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    queued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ NULL,
    completed_at TIMESTAMPTZ NULL,
    failed_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE generation_jobs
    ADD COLUMN IF NOT EXISTS request_fingerprint TEXT NOT NULL DEFAULT '';

ALTER TABLE generation_jobs
    ADD COLUMN IF NOT EXISTS request_payload JSONB NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE generation_jobs
    ADD COLUMN IF NOT EXISTS workout_id UUID NULL;

ALTER TABLE generation_jobs
    ADD COLUMN IF NOT EXISTS error TEXT NULL;

ALTER TABLE generation_jobs
    ADD COLUMN IF NOT EXISTS version BIGINT NOT NULL DEFAULT 1;

ALTER TABLE generation_jobs
    ADD COLUMN IF NOT EXISTS queued_at TIMESTAMPTZ NOT NULL DEFAULT now();

ALTER TABLE generation_jobs
    ADD COLUMN IF NOT EXISTS started_at TIMESTAMPTZ NULL;

ALTER TABLE generation_jobs
    ADD COLUMN IF NOT EXISTS completed_at TIMESTAMPTZ NULL;

ALTER TABLE generation_jobs
    ADD COLUMN IF NOT EXISTS failed_at TIMESTAMPTZ NULL;

CREATE INDEX IF NOT EXISTS generation_jobs_user_status_idx
    ON generation_jobs (user_id, status);

CREATE INDEX IF NOT EXISTS generation_jobs_user_created_at_idx
    ON generation_jobs (user_id, created_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS generation_jobs_active_dedupe_idx
    ON generation_jobs (user_id, date, request_fingerprint)
    WHERE status IN ('queued', 'running');

CREATE TABLE IF NOT EXISTS workout_preferences (
    user_id BIGINT PRIMARY KEY,
    max_sets_per_exercise SMALLINT,
    preferred_split TEXT,
    training_goal TEXT,
    session_duration_minutes INTEGER,
    notes TEXT,
    CHECK (max_sets_per_exercise IS NULL OR max_sets_per_exercise >= 0),
    CHECK (session_duration_minutes IS NULL OR session_duration_minutes >= 0)
);

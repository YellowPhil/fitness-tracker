use std::sync::{Arc, Mutex};

use postgres::{Client, NoTls};

pub(crate) type SharedClient = Arc<Mutex<Client>>;

pub(crate) fn connect(url: &str) -> Result<SharedClient, postgres::Error> {
    let mut client = Client::connect(url, NoTls)?;
    init_schema(&mut client)?;
    Ok(Arc::new(Mutex::new(client)))
}

fn init_schema(client: &mut Client) -> Result<(), postgres::Error> {
    client.batch_execute(
        "
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
            PRIMARY KEY (id, user_id)
        );

        CREATE INDEX IF NOT EXISTS workouts_user_id_start_date_idx
            ON workouts (user_id, start_date DESC);

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
        ",
    )?;

    Ok(())
}

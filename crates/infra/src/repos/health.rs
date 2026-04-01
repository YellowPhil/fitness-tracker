use std::path::Path;

use domain::{
    health::HealthParams,
    traits::HealthRepo,
    types::{Height, HeightUnits, UserId, Weight, WeightUnits},
};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, thiserror::Error)]
pub enum SqliteHealthRepoError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

pub struct SqliteHealthDb {
    connection: Connection,
}

pub struct SqliteHealthRepo<'db> {
    connection: &'db Connection,
    user_id: UserId,
}

impl SqliteHealthDb {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SqliteHealthRepoError> {
        let connection = Connection::open(path)?;
        Self::init_schema(&connection)?;
        Ok(Self { connection })
    }

    pub fn in_memory() -> Result<Self, SqliteHealthRepoError> {
        let connection = Connection::open_in_memory()?;
        Self::init_schema(&connection)?;
        Ok(Self { connection })
    }

    pub fn for_user(&self, user_id: UserId) -> SqliteHealthRepo<'_> {
        SqliteHealthRepo {
            connection: &self.connection,
            user_id,
        }
    }

    fn init_schema(connection: &Connection) -> Result<(), SqliteHealthRepoError> {
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS health_params (
                user_id INTEGER PRIMARY KEY,
                weight_value REAL NOT NULL,
                weight_units INTEGER NOT NULL,
                height_value REAL NOT NULL,
                height_units INTEGER NOT NULL,
                age INTEGER NOT NULL
            );",
        )?;
        Ok(())
    }
}

impl HealthRepo for SqliteHealthRepo<'_> {
    type RepoError = SqliteHealthRepoError;

    fn get_health(&self) -> Result<HealthParams, Self::RepoError> {
        let row = self
            .connection
            .query_row(
                "SELECT weight_value, weight_units, height_value, height_units, age
                 FROM health_params
                 WHERE user_id = ?1",
                params![self.user_id],
                |row| {
                    Ok(HealthParams::new(
                        Height::new(row.get::<_, f64>(2)?, row.get::<_, HeightUnits>(3)?),
                        Weight::new(row.get::<_, f64>(0)?, row.get::<_, WeightUnits>(1)?),
                        row.get::<_, u32>(4)?,
                    ))
                },
            )
            .optional()?;

        Ok(row.unwrap_or_else(|| {
            HealthParams::new(
                Height::new(170.0, HeightUnits::Centimeters),
                Weight::new(70.0, WeightUnits::Kilograms),
                25,
            )
        }))
    }

    fn save(&self, params: &HealthParams) -> Result<(), Self::RepoError> {
        self.connection.execute(
            "INSERT INTO health_params (user_id, weight_value, weight_units, height_value, height_units, age)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(user_id) DO UPDATE SET
                 weight_value = excluded.weight_value,
                 weight_units = excluded.weight_units,
                 height_value = excluded.height_value,
                 height_units = excluded.height_units,
                 age = excluded.age",
            params![
                self.user_id,
                params.weight.value,
                params.weight.units,
                params.height.value,
                params.height.units,
                params.age,
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use domain::{
        health::HealthParams,
        traits::HealthRepo,
        types::{Height, HeightUnits, UserId, Weight, WeightUnits},
    };

    use super::SqliteHealthDb;

    #[test]
    fn returns_defaults_when_no_row() {
        let db = SqliteHealthDb::in_memory().expect("db should initialize");
        let repo = db.for_user(UserId::new(1));
        let params = repo.get_health().expect("should succeed");
        assert_eq!(params.weight.value, 70.0);
        assert_eq!(params.weight.units, WeightUnits::Kilograms);
        assert_eq!(params.height.value, 170.0);
        assert_eq!(params.height.units, HeightUnits::Centimeters);
        assert_eq!(params.age, 25);
    }

    #[test]
    fn saves_and_loads_health_params() {
        let db = SqliteHealthDb::in_memory().expect("db should initialize");
        let repo = db.for_user(UserId::new(1));

        let params = HealthParams::new(
            Height::new(180.0, HeightUnits::Centimeters),
            Weight::new(85.5, WeightUnits::Kilograms),
            30,
        );
        repo.save(&params).expect("save should succeed");

        let loaded = repo.get_health().expect("load should succeed");
        assert_eq!(loaded.weight.value, 85.5);
        assert_eq!(loaded.weight.units, WeightUnits::Kilograms);
        assert_eq!(loaded.height.value, 180.0);
        assert_eq!(loaded.height.units, HeightUnits::Centimeters);
        assert_eq!(loaded.age, 30);
    }

    #[test]
    fn different_users_see_own_params() {
        let db = SqliteHealthDb::in_memory().expect("db should initialize");
        let repo_a = db.for_user(UserId::new(1));
        let repo_b = db.for_user(UserId::new(2));

        let params = HealthParams::new(
            Height::new(175.0, HeightUnits::Centimeters),
            Weight::new(90.0, WeightUnits::Pounds),
            28,
        );
        repo_a.save(&params).expect("save should succeed");

        let loaded_a = repo_a.get_health().unwrap();
        assert_eq!(loaded_a.weight.value, 90.0);

        let loaded_b = repo_b.get_health().unwrap();
        assert_eq!(loaded_b.weight.value, 70.0); // default
    }

    #[test]
    fn upserts_on_conflict() {
        let db = SqliteHealthDb::in_memory().expect("db should initialize");
        let repo = db.for_user(UserId::new(1));

        let v1 = HealthParams::new(
            Height::new(170.0, HeightUnits::Centimeters),
            Weight::new(70.0, WeightUnits::Kilograms),
            25,
        );
        repo.save(&v1).expect("first save");

        let v2 = HealthParams::new(
            Height::new(170.0, HeightUnits::Centimeters),
            Weight::new(72.0, WeightUnits::Kilograms),
            26,
        );
        repo.save(&v2).expect("second save");

        let loaded = repo.get_health().unwrap();
        assert_eq!(loaded.weight.value, 72.0);
        assert_eq!(loaded.age, 26);
    }
}

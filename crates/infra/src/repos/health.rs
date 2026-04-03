use std::sync::{MutexGuard, PoisonError};

use domain::{
    health::HealthParams,
    traits::HealthRepo,
    types::{Height, HeightUnits, UserId, Weight, WeightUnits},
};
use postgres::Client;

use super::{
    postgres::{SharedClient, connect},
    postgres_types::{PgHeightUnits, PgWeightUnits},
};

#[derive(Debug, thiserror::Error)]
pub enum PostgresHealthRepoError {
    #[error("postgres error: {0}")]
    Postgres(#[from] postgres::Error),
    #[error("postgres connection lock poisoned")]
    ConnectionPoisoned,
    #[error("age value out of range for domain type: {0}")]
    InvalidAge(i32),
    #[error("age value exceeds supported range: {0}")]
    AgeOutOfRange(u32),
}

pub struct PostgresHealthDb {
    client: SharedClient,
}

pub struct PostgresHealthRepo<'db> {
    client: &'db SharedClient,
    user_id: UserId,
}

impl PostgresHealthDb {
    pub fn open(url: &str) -> Result<Self, PostgresHealthRepoError> {
        Ok(Self {
            client: connect(url)?,
        })
    }

    pub(crate) fn new(client: SharedClient) -> Self {
        Self { client }
    }

    pub fn for_user(&self, user_id: UserId) -> PostgresHealthRepo<'_> {
        PostgresHealthRepo {
            client: &self.client,
            user_id,
        }
    }
}

impl PostgresHealthRepo<'_> {
    fn client(&self) -> Result<MutexGuard<'_, Client>, PostgresHealthRepoError> {
        self.client
            .lock()
            .map_err(|_: PoisonError<MutexGuard<'_, Client>>| {
                PostgresHealthRepoError::ConnectionPoisoned
            })
    }
}

impl HealthRepo for PostgresHealthRepo<'_> {
    type RepoError = PostgresHealthRepoError;

    fn get_health(&self) -> Result<HealthParams, Self::RepoError> {
        let mut client = self.client()?;
        let row = client.query_opt(
            "SELECT weight_value, weight_units, height_value, height_units, age
             FROM health_params
             WHERE user_id = $1",
            &[&self.user_id.as_i64()],
        )?;

        row.map(health_from_row).transpose().map(|params| {
            params.unwrap_or_else(|| {
                HealthParams::new(
                    Height::new(170.0, HeightUnits::Centimeters),
                    Weight::new(70.0, WeightUnits::Kilograms),
                    25,
                )
            })
        })
    }

    fn save(&self, params: &HealthParams) -> Result<(), Self::RepoError> {
        let age = i32::try_from(params.age)
            .map_err(|_| PostgresHealthRepoError::AgeOutOfRange(params.age))?;
        let mut client = self.client()?;

        client.execute(
            "INSERT INTO health_params (
                user_id,
                weight_value,
                weight_units,
                height_value,
                height_units,
                age
             )
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (user_id) DO UPDATE SET
                weight_value = EXCLUDED.weight_value,
                weight_units = EXCLUDED.weight_units,
                height_value = EXCLUDED.height_value,
                height_units = EXCLUDED.height_units,
                age = EXCLUDED.age",
            &[
                &self.user_id.as_i64(),
                &params.weight.value,
                &PgWeightUnits::from(params.weight.units),
                &params.height.value,
                &PgHeightUnits::from(params.height.units),
                &age,
            ],
        )?;

        Ok(())
    }
}

fn health_from_row(row: postgres::Row) -> Result<HealthParams, PostgresHealthRepoError> {
    let age: i32 = row.get("age");

    Ok(HealthParams::new(
        Height::new(
            row.get("height_value"),
            HeightUnits::from(row.get::<_, PgHeightUnits>("height_units")),
        ),
        Weight::new(
            row.get("weight_value"),
            WeightUnits::from(row.get::<_, PgWeightUnits>("weight_units")),
        ),
        u32::try_from(age).map_err(|_| PostgresHealthRepoError::InvalidAge(age))?,
    ))
}

#[cfg(test)]
mod tests {
    use domain::types::{HeightUnits, WeightUnits};

    use crate::repos::postgres_types::{PgHeightUnits, PgWeightUnits};

    #[test]
    fn postgres_unit_enums_match_domain_units() {
        assert_eq!(
            PgWeightUnits::from(WeightUnits::Kilograms),
            PgWeightUnits::Kilograms
        );
        assert_eq!(WeightUnits::from(PgWeightUnits::Pounds), WeightUnits::Pounds);
        assert_eq!(
            PgHeightUnits::from(HeightUnits::Centimeters),
            PgHeightUnits::Centimeters
        );
        assert_eq!(HeightUnits::from(PgHeightUnits::Inches), HeightUnits::Inches);
    }
}

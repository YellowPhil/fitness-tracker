use domain::{
    health::HealthParams,
    traits::HealthRepo,
    types::{Height, HeightUnits, UserId, Weight, WeightUnits},
};
use sqlx::{Pool, Postgres, Row, postgres::PgRow};

use super::postgres_types::{PgHeightUnits, PgWeightUnits};

#[derive(Debug, thiserror::Error)]
pub enum PostgresHealthRepoError {
    #[error("postgres error: {0}")]
    Postgres(#[from] sqlx::Error),
    #[error("age value out of range for domain type: {0}")]
    InvalidAge(i32),
    #[error("age value exceeds supported range: {0}")]
    AgeOutOfRange(u32),
}

pub struct PostgresHealthDb {
    pool: Pool<Postgres>,
}

impl PostgresHealthDb {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub fn for_user(&self, user_id: UserId) -> PostgresHealthRepo {
        PostgresHealthRepo {
            pool: self.pool.clone(),
            user_id,
        }
    }
}

pub struct PostgresHealthRepo {
    pool: Pool<Postgres>,
    user_id: UserId,
}

#[async_trait::async_trait]
impl HealthRepo for PostgresHealthRepo {
    type RepoError = PostgresHealthRepoError;

    async fn get_health(&self) -> Result<HealthParams, Self::RepoError> {
        let row = sqlx::query(
            "SELECT weight_value, weight_units, height_value, height_units, age
             FROM health_params
             WHERE user_id = $1",
        )
        .bind(self.user_id.as_i64())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row
            .map(health_from_row)
            .transpose()?
            .unwrap_or_else(|| {
                HealthParams::new(
                    Height::new(170.0, HeightUnits::Centimeters),
                    Weight::new(70.0, WeightUnits::Kilograms),
                    25,
                )
            }))
    }

    async fn save(&self, params: &HealthParams) -> Result<(), Self::RepoError> {
        let age = i32::try_from(params.age)
            .map_err(|_| PostgresHealthRepoError::AgeOutOfRange(params.age))?;

        sqlx::query(
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
        )
        .bind(self.user_id.as_i64())
        .bind(params.weight.value)
        .bind(PgWeightUnits::from(params.weight.units))
        .bind(params.height.value)
        .bind(PgHeightUnits::from(params.height.units))
        .bind(age)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

fn health_from_row(row: PgRow) -> Result<HealthParams, PostgresHealthRepoError> {
    let age: i32 = row.get("age");

    Ok(HealthParams::new(
        Height::new(
            row.get("height_value"),
            HeightUnits::from(row.get::<PgHeightUnits, _>("height_units")),
        ),
        Weight::new(
            row.get("weight_value"),
            WeightUnits::from(row.get::<PgWeightUnits, _>("weight_units")),
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

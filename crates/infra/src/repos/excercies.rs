use domain::{
    excercise::{Exercise, ExerciseId, ExerciseKind, ExerciseMetadata, MuscleGroup},
    traits::ExcerciseRepo,
    types::UserId,
};
use sqlx::{Pool, Postgres, Row, postgres::PgRow};

use super::postgres_types::{
    PgExerciseKind, PgExerciseSource, PgMuscleGroup, from_pg_muscle_groups, to_pg_muscle_groups,
};

#[derive(Debug, thiserror::Error)]
pub enum PostgresExcerciseRepoError {
    #[error("postgres error: {0}")]
    Postgres(#[from] sqlx::Error),
}

pub struct PostgresExcerciseDb {
    pool: Pool<Postgres>,
}

impl PostgresExcerciseDb {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub fn for_user(&self, user_id: UserId) -> PostgresExcerciseRepo {
        PostgresExcerciseRepo {
            pool: self.pool.clone(),
            user_id,
        }
    }
}

pub struct PostgresExcerciseRepo {
    pool: Pool<Postgres>,
    user_id: UserId,
}

#[async_trait::async_trait]
impl ExcerciseRepo for PostgresExcerciseRepo {
    type RepoError = PostgresExcerciseRepoError;

    async fn get_by_id(&self, id: &ExerciseId) -> Result<Option<Exercise>, Self::RepoError> {
        let row = sqlx::query(
            "SELECT id, name, kind, muscle_group, secondary_muscle_groups, source
             FROM exercises
             WHERE id = $1 AND user_id = $2",
        )
        .bind(id.as_uuid())
        .bind(self.user_id.as_i64())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(exercise_from_row))
    }

    async fn save(&self, exercise: &Exercise) -> Result<(), Self::RepoError> {
        let secondary_muscle_groups = to_pg_muscle_groups(&exercise.secondary_muscle_groups);

        sqlx::query(
            "INSERT INTO exercises (
                id,
                user_id,
                name,
                kind,
                muscle_group,
                secondary_muscle_groups,
                source
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (id, user_id) DO UPDATE SET
                name = EXCLUDED.name,
                kind = EXCLUDED.kind,
                muscle_group = EXCLUDED.muscle_group,
                secondary_muscle_groups = EXCLUDED.secondary_muscle_groups,
                source = EXCLUDED.source",
        )
        .bind(exercise.id.as_uuid())
        .bind(self.user_id.as_i64())
        .bind(&exercise.name)
        .bind(PgExerciseKind::from(exercise.kind))
        .bind(PgMuscleGroup::from(exercise.muscle_group))
        .bind(secondary_muscle_groups)
        .bind(PgExerciseSource::from(exercise.source))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_muscle_group(
        &self,
        muscle_group: MuscleGroup,
    ) -> Result<Vec<Exercise>, Self::RepoError> {
        let rows = sqlx::query(
            "SELECT id, name, kind, muscle_group, secondary_muscle_groups, source
             FROM exercises
             WHERE user_id = $1 AND muscle_group = $2
             ORDER BY name ASC",
        )
        .bind(self.user_id.as_i64())
        .bind(PgMuscleGroup::from(muscle_group))
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(exercise_from_row).collect())
    }

    async fn get_all(&self) -> Result<Vec<Exercise>, Self::RepoError> {
        let rows = sqlx::query(
            "SELECT id, name, kind, muscle_group, secondary_muscle_groups, source
             FROM exercises
             WHERE user_id = $1
             ORDER BY name ASC",
        )
        .bind(self.user_id.as_i64())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(exercise_from_row).collect())
    }

    async fn get_metadata_by_ids(
        &self,
        ids: &[ExerciseId],
    ) -> Result<Vec<ExerciseMetadata>, Self::RepoError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let uuids: Vec<_> = ids.iter().map(|id| *id.as_uuid()).collect();
        let rows = sqlx::query(
            "SELECT id, name, muscle_group, secondary_muscle_groups
             FROM exercises
             WHERE user_id = $1 AND id = ANY($2)
             ORDER BY name ASC",
        )
        .bind(self.user_id.as_i64())
        .bind(&uuids[..])
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(metadata_from_row).collect())
    }

    async fn delete(&self, id: &ExerciseId) -> Result<(), Self::RepoError> {
        sqlx::query("DELETE FROM exercises WHERE id = $1 AND user_id = $2")
            .bind(id.as_uuid())
            .bind(self.user_id.as_i64())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

fn exercise_from_row(row: PgRow) -> Exercise {
    Exercise {
        id: ExerciseId::from_uuid(row.get("id")),
        name: row.get("name"),
        kind: ExerciseKind::from(row.get::<PgExerciseKind, _>("kind")),
        muscle_group: row.get::<PgMuscleGroup, _>("muscle_group").into(),
        secondary_muscle_groups: from_pg_muscle_groups(row.get("secondary_muscle_groups")),
        source: row.get::<PgExerciseSource, _>("source").into(),
    }
}

fn metadata_from_row(row: PgRow) -> ExerciseMetadata {
    ExerciseMetadata {
        id: ExerciseId::from_uuid(row.get("id")),
        name: row.get("name"),
        muscle_group: row.get::<PgMuscleGroup, _>("muscle_group").into(),
        secondary_muscle_groups: from_pg_muscle_groups(row.get("secondary_muscle_groups")),
    }
}

#[cfg(test)]
mod tests {
    use domain::excercise::{ExerciseKind, ExerciseSource, MuscleGroup};

    use crate::repos::postgres_types::{PgExerciseKind, PgExerciseSource, PgMuscleGroup};

    #[test]
    fn postgres_enums_match_domain_enums() {
        assert_eq!(
            PgExerciseKind::from(ExerciseKind::Weighted),
            PgExerciseKind::Weighted
        );
        assert_eq!(
            ExerciseKind::from(PgExerciseKind::BodyWeight),
            ExerciseKind::BodyWeight
        );
        assert_eq!(
            PgExerciseSource::from(ExerciseSource::BuiltIn),
            PgExerciseSource::BuiltIn
        );
        assert_eq!(
            ExerciseSource::from(PgExerciseSource::UserDefined),
            ExerciseSource::UserDefined
        );
        assert_eq!(
            PgMuscleGroup::from(MuscleGroup::Chest),
            PgMuscleGroup::Chest
        );
        assert_eq!(MuscleGroup::from(PgMuscleGroup::Core), MuscleGroup::Core);
    }
}

use std::sync::{MutexGuard, PoisonError};

use domain::{
    excercise::{Exercise, ExerciseId, ExerciseKind, ExerciseMetadata},
    traits::ExcerciseRepo,
    types::UserId,
};
use postgres::{Client, Row};

use super::{
    postgres::{SharedClient, connect},
    postgres_types::{
        PgExerciseKind, PgExerciseSource, PgMuscleGroup, from_pg_muscle_groups, to_pg_muscle_groups,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum PostgresExcerciseRepoError {
    #[error("postgres error: {0}")]
    Postgres(#[from] postgres::Error),
    #[error("postgres connection lock poisoned")]
    ConnectionPoisoned,
}

pub struct PostgresExcerciseDb {
    client: SharedClient,
}

pub struct PostgresExcerciseRepo<'db> {
    client: &'db SharedClient,
    user_id: UserId,
}

impl PostgresExcerciseDb {
    pub fn open(url: &str) -> Result<Self, PostgresExcerciseRepoError> {
        Ok(Self {
            client: connect(url)?,
        })
    }

    pub(crate) fn new(client: SharedClient) -> Self {
        Self { client }
    }

    pub fn for_user(&self, user_id: UserId) -> PostgresExcerciseRepo<'_> {
        PostgresExcerciseRepo {
            client: &self.client,
            user_id,
        }
    }
}

impl PostgresExcerciseRepo<'_> {
    fn client(&self) -> Result<MutexGuard<'_, Client>, PostgresExcerciseRepoError> {
        self.client
            .lock()
            .map_err(|_: PoisonError<MutexGuard<'_, Client>>| {
                PostgresExcerciseRepoError::ConnectionPoisoned
            })
    }
}

impl ExcerciseRepo for PostgresExcerciseRepo<'_> {
    type RepoError = PostgresExcerciseRepoError;

    fn get_by_id(&self, id: &ExerciseId) -> Result<Option<Exercise>, Self::RepoError> {
        let mut client = self.client()?;
        let row = client.query_opt(
            "SELECT id, name, kind, muscle_group, secondary_muscle_groups, source
             FROM exercises
             WHERE id = $1 AND user_id = $2",
            &[id.as_uuid(), &self.user_id.as_i64()],
        )?;

        Ok(row.map(exercise_from_row))
    }

    fn save(&self, exercise: &Exercise) -> Result<(), Self::RepoError> {
        let mut client = self.client()?;
        let secondary_muscle_groups = to_pg_muscle_groups(&exercise.secondary_muscle_groups);

        client.execute(
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
            &[
                exercise.id.as_uuid(),
                &self.user_id.as_i64(),
                &exercise.name,
                &PgExerciseKind::from(exercise.kind),
                &PgMuscleGroup::from(exercise.muscle_group),
                &secondary_muscle_groups,
                &PgExerciseSource::from(exercise.source),
            ],
        )?;

        Ok(())
    }

    fn get_by_muscle_group(
        &self,
        muscle_group: MuscleGroup,
    ) -> Result<Vec<Exercise>, Self::RepoError> {
        let mut client = self.client()?;
        let rows = client.query(
            "SELECT id, name, kind, muscle_group, secondary_muscle_groups, source
             FROM exercises
             WHERE user_id = $1 AND muscle_group = $2
             ORDER BY name ASC",
            &[&self.user_id.as_i64(), &PgMuscleGroup::from(muscle_group)],
        )?;

        Ok(rows.into_iter().map(exercise_from_row).collect())
    }

    fn get_all(&self) -> Result<Vec<Exercise>, Self::RepoError> {
        let mut client = self.client()?;
        let rows = client.query(
            "SELECT id, name, kind, muscle_group, secondary_muscle_groups, source
             FROM exercises
             WHERE user_id = $1
             ORDER BY name ASC",
            &[&self.user_id.as_i64()],
        )?;

        Ok(rows.into_iter().map(exercise_from_row).collect())
    }

    fn get_metadata_by_ids(
        &self,
        ids: &[ExerciseId],
    ) -> Result<Vec<ExerciseMetadata>, Self::RepoError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let ids: Vec<_> = ids.iter().map(|id| *id.as_uuid()).collect();
        let mut client = self.client()?;
        let rows = client.query(
            "SELECT id, name, muscle_group, secondary_muscle_groups
             FROM exercises
             WHERE user_id = $1 AND id = ANY($2)
             ORDER BY name ASC",
            &[&self.user_id.as_i64(), &ids],
        )?;

        Ok(rows.into_iter().map(metadata_from_row).collect())
    }

    fn delete(&self, id: &ExerciseId) -> Result<(), Self::RepoError> {
        let mut client = self.client()?;
        client.execute(
            "DELETE FROM exercises WHERE id = $1 AND user_id = $2",
            &[id.as_uuid(), &self.user_id.as_i64()],
        )?;
        Ok(())
    }
}

fn exercise_from_row(row: Row) -> Exercise {
    Exercise {
        id: ExerciseId::from_uuid(row.get("id")),
        name: row.get("name"),
        kind: ExerciseKind::from(row.get::<_, PgExerciseKind>("kind")),
        muscle_group: row.get::<_, PgMuscleGroup>("muscle_group").into(),
        secondary_muscle_groups: from_pg_muscle_groups(row.get("secondary_muscle_groups")),
        source: row.get::<_, PgExerciseSource>("source").into(),
    }
}

fn metadata_from_row(row: Row) -> ExerciseMetadata {
    ExerciseMetadata {
        id: ExerciseId::from_uuid(row.get("id")),
        name: row.get("name"),
        muscle_group: row.get::<_, PgMuscleGroup>("muscle_group").into(),
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

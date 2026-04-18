use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use domain::types::{
    ExerciseId, LoadType, MuscleGroup, PerformedSet, Weight, WeightUnits, Workout, WorkoutExercise,
};
use fitness_tracker_proto::common::MuscleGroup as ProtoMuscleGroup;
use fitness_tracker_proto::workout_generator::GenerateWorkoutRequest as GenerateWorkoutGrpcRequest;
use tracing::{error, info};

use crate::generation::{GenerationDispatcher, GenerationPayload, parse_generation_payload};
use crate::repos::generation_jobs::{GenerationJob, PostgresGenerationJobDb};
use crate::web::Databases;

use super::event_bus::GenerationEventBus;

#[derive(Clone)]
pub struct InProcessGenerationDispatcher {
    pub databases: Arc<Databases>,
    pub generation_jobs_db: PostgresGenerationJobDb,
    pub event_bus: GenerationEventBus,
    pub workout_generator_grpc_addr: String,
    pub grpc_timeout: std::time::Duration,
}

#[async_trait]
impl GenerationDispatcher for InProcessGenerationDispatcher {
    async fn dispatch(&self, job: GenerationJob) -> anyhow::Result<()> {
        let this = self.clone();
        tokio::spawn(async move {
            if let Err(err) = this.execute(job).await {
                error!(error = %err, "generation worker failed");
            }
        });
        Ok(())
    }
}

impl InProcessGenerationDispatcher {
    async fn execute(&self, job: GenerationJob) -> anyhow::Result<()> {
        let repo = self.generation_jobs_db.for_user(job.user_id);
        let running_job = match repo.mark_running(job.id).await? {
            Some(job) => job,
            None => return Ok(()),
        };

        self.event_bus.publish(running_job.clone());

        let payload = parse_generation_payload(&running_job.request_payload)
            .context("parse generation payload")?;

        let result = self
            .generate_and_persist_workout(running_job.user_id, &payload)
            .await;

        match result {
            Ok(workout_id) => {
                let completed_job = repo.mark_completed(running_job.id, workout_id).await?;
                self.event_bus.publish(completed_job);
                Ok(())
            }
            Err(err) => {
                let message = err.to_string();
                let failed_job = repo.mark_failed(running_job.id, &message).await?;
                self.event_bus.publish(failed_job);
                Err(err)
            }
        }
    }

    async fn generate_and_persist_workout(
        &self,
        user_id: domain::types::UserId,
        payload: &GenerationPayload,
    ) -> anyhow::Result<domain::types::WorkoutId> {
        let app = self.databases.gym_app(user_id);
        app.seed_built_in_excercises().await?;

        let muscle_groups = payload
            .muscle_groups
            .iter()
            .map(|name| MuscleGroup::from_str(name))
            .collect::<Result<Vec<_>, _>>()
            .context("parse muscle groups from payload")?;

        let max_exercise_count =
            i32::try_from(payload.max_exercise_count).context("max_exercise_count exceeds i32")?;

        let grpc_request = GenerateWorkoutGrpcRequest {
            user_id: user_id.as_i64(),
            date: payload.start_date.date().to_string(),
            muscle_groups: muscle_groups
                .iter()
                .map(|group| proto_muscle_group(*group) as i32)
                .collect(),
            max_exercise_count,
        };

        let generated = crate::grpc::request_generated_workout(
            &self.workout_generator_grpc_addr,
            self.grpc_timeout,
            grpc_request,
        )
        .await?;

        let entries = generated
            .exercises
            .into_iter()
            .map(map_generated_exercise)
            .collect::<Result<Vec<_>, _>>()?;

        let workout = Workout::ai_generated(generated.workout_name, payload.start_date, entries);

        self.databases
            .gym_app(user_id)
            .save_workout(&workout)
            .await?;

        info!(
            user_id = user_id.as_i64(),
            workout_id = %workout.id.as_uuid(),
            "generation job completed"
        );

        Ok(workout.id)
    }
}

fn map_generated_exercise(
    exercise: fitness_tracker_proto::workout_generator::GeneratedExercise,
) -> anyhow::Result<WorkoutExercise> {
    let exercise_id = ExerciseId::from_uuid(uuid::Uuid::parse_str(&exercise.exercise_id)?);

    let sets = exercise
        .sets
        .into_iter()
        .map(|set| {
            let reps =
                u32::try_from(set.reps).context("generated set reps must be non-negative")?;
            let kind = match set.weight_kg {
                Some(weight_kg) if weight_kg > 0.0 => {
                    LoadType::Weighted(Weight::new(weight_kg, WeightUnits::Kilograms))
                }
                _ => LoadType::BodyWeight,
            };
            Ok(PerformedSet { reps, kind })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(WorkoutExercise {
        exercise_id,
        sets,
        notes: exercise.notes,
    })
}

fn proto_muscle_group(value: MuscleGroup) -> ProtoMuscleGroup {
    match value {
        MuscleGroup::Chest => ProtoMuscleGroup::Chest,
        MuscleGroup::Back => ProtoMuscleGroup::Back,
        MuscleGroup::Shoulders => ProtoMuscleGroup::Shoulders,
        MuscleGroup::Arms => ProtoMuscleGroup::Arms,
        MuscleGroup::Legs => ProtoMuscleGroup::Legs,
        MuscleGroup::Core => ProtoMuscleGroup::Core,
    }
}

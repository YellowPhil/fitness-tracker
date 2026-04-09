use std::collections::HashMap;
use std::sync::Arc;

use domain::traits::ExcerciseRepo;
use domain::types::{
    Exercise, ExerciseKind, ExerciseMetadata, MuscleGroup, QueryType, UserId, WorkoutQuery,
};
use fitness_tracker_proto::common::{
    ExerciseCatalogItem as ProtoExerciseCatalogItem, ExerciseKind as ProtoExerciseKind,
    MuscleGroup as ProtoMuscleGroup,
};
use fitness_tracker_proto::workout_data::workout_data_service_server::WorkoutDataService;
use fitness_tracker_proto::workout_data::{
    GetExerciseCatalogRequest, GetExerciseCatalogResponse, ListExercisesRequest,
    ListExercisesResponse, QueryWorkoutsRequest, QueryWorkoutsResponse,
};
use tonic::{Request, Response, Status};
use tracing::instrument;

use crate::web::Databases;

pub struct WorkoutDataGrpcService {
    databases: Arc<Databases>,
}

impl WorkoutDataGrpcService {
    pub fn new(databases: Arc<Databases>) -> Self {
        Self { databases }
    }
}

#[tonic::async_trait]
impl WorkoutDataService for WorkoutDataGrpcService {
    #[instrument(skip(self, request), err)]
    async fn get_exercise_catalog(
        &self,
        request: Request<GetExerciseCatalogRequest>,
    ) -> Result<Response<GetExerciseCatalogResponse>, Status> {
        let payload = request.into_inner();
        if payload.muscle_groups.is_empty() {
            return Err(Status::invalid_argument("muscle_groups must not be empty"));
        }

        let user_id = UserId::new(payload.user_id);
        let app = self.databases.gym_app(user_id);
        app.seed_built_in_excercises()
            .await
            .map_err(internal_status)?;

        let mut by_id: HashMap<String, ProtoExerciseCatalogItem> = HashMap::new();
        for value in payload.muscle_groups {
            let muscle_group = muscle_group_from_proto(value)?;
            let exercises = self
                .databases
                .exercise_db
                .for_user(user_id)
                .get_by_muscle_group(muscle_group)
                .await
                .map_err(internal_status)?;

            for exercise in exercises {
                let item = exercise_catalog_item_to_proto(exercise);
                by_id.insert(item.exercise_id.clone(), item);
            }
        }

        Ok(Response::new(GetExerciseCatalogResponse {
            exercises: by_id.into_values().collect(),
        }))
    }

    #[instrument(skip(self, request), err)]
    async fn query_workouts(
        &self,
        request: Request<QueryWorkoutsRequest>,
    ) -> Result<Response<QueryWorkoutsResponse>, Status> {
        let payload = request.into_inner();
        if payload.date.is_some() && payload.last_n.is_some() {
            return Err(Status::invalid_argument(
                "date and last_n are mutually exclusive",
            ));
        }

        let user_id = UserId::new(payload.user_id);
        let muscle_group = muscle_group_from_proto(payload.muscle_group)?;
        let date = match payload.date {
            Some(raw_date) => QueryType::OnDate(parse_date_yyyy_mm_dd(&raw_date)?),
            None => match payload.last_n {
                Some(last_n) if last_n > 0 => QueryType::LastN(last_n as usize),
                Some(_) => {
                    return Err(Status::invalid_argument("last_n must be greater than 0"));
                }
                None => QueryType::Latest,
            },
        };

        let result = self
            .databases
            .gym_app(user_id)
            .query_workout_resource(WorkoutQuery {
                date,
                muscle_group: Some(muscle_group),
            })
            .await
            .map_err(internal_status)?;

        let content = crate::ai::format::format_workouts(
            &result.workouts,
            &result.excercises,
            Some(muscle_group),
        );

        Ok(Response::new(QueryWorkoutsResponse { content }))
    }

    #[instrument(skip(self, request), err)]
    async fn list_exercises(
        &self,
        request: Request<ListExercisesRequest>,
    ) -> Result<Response<ListExercisesResponse>, Status> {
        let payload = request.into_inner();
        let user_id = UserId::new(payload.user_id);
        let app = self.databases.gym_app(user_id);
        app.seed_built_in_excercises()
            .await
            .map_err(internal_status)?;

        let muscle_group = muscle_group_from_proto(payload.muscle_group)?;
        let exercises = self
            .databases
            .exercise_db
            .for_user(user_id)
            .get_by_muscle_group(muscle_group)
            .await
            .map_err(internal_status)?;

        let metadata = exercises.iter().map(Exercise::metadata).collect::<Vec<_>>();
        let content = format_exercises(&metadata, muscle_group);

        Ok(Response::new(ListExercisesResponse { content }))
    }
}

fn exercise_catalog_item_to_proto(exercise: Exercise) -> ProtoExerciseCatalogItem {
    ProtoExerciseCatalogItem {
        exercise_id: exercise.id.as_uuid().to_string(),
        name: exercise.name,
        kind: proto_exercise_kind(exercise.kind) as i32,
        muscle_group: proto_muscle_group(exercise.muscle_group) as i32,
    }
}

fn format_exercises(exercises: &[ExerciseMetadata], muscle_group: MuscleGroup) -> String {
    crate::ai::format::format_exercises(exercises, Some(muscle_group))
}

fn muscle_group_from_proto(value: i32) -> Result<MuscleGroup, Status> {
    if value == 0 {
        return Err(Status::invalid_argument("muscle_group must be specified"));
    }

    let proto = ProtoMuscleGroup::try_from(value)
        .map_err(|_| Status::invalid_argument("invalid muscle_group value"))?;

    match proto {
        ProtoMuscleGroup::Chest => Ok(MuscleGroup::Chest),
        ProtoMuscleGroup::Back => Ok(MuscleGroup::Back),
        ProtoMuscleGroup::Shoulders => Ok(MuscleGroup::Shoulders),
        ProtoMuscleGroup::Arms => Ok(MuscleGroup::Arms),
        ProtoMuscleGroup::Legs => Ok(MuscleGroup::Legs),
        ProtoMuscleGroup::Core => Ok(MuscleGroup::Core),
        _ => Err(Status::invalid_argument("invalid muscle_group value")),
    }
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

fn proto_exercise_kind(value: ExerciseKind) -> ProtoExerciseKind {
    match value {
        ExerciseKind::Weighted => ProtoExerciseKind::Weighted,
        ExerciseKind::BodyWeight => ProtoExerciseKind::BodyWeight,
    }
}

fn parse_date_yyyy_mm_dd(input: &str) -> Result<time::Date, Status> {
    let format = time::format_description::parse_borrowed::<2>("[year]-[month]-[day]")
        .map_err(|e| Status::internal(e.to_string()))?;
    time::Date::parse(input, &format)
        .map_err(|_| Status::invalid_argument(format!("invalid date format: {input}")))
}

fn internal_status(error: impl std::fmt::Display) -> Status {
    Status::internal(error.to_string())
}

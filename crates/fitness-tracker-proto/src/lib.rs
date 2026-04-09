pub mod common {
    tonic::include_proto!("fitness_tracker.common");
}

pub mod workout_data {
    tonic::include_proto!("fitness_tracker.workout_data");
}

pub mod workout_generator {
    tonic::include_proto!("fitness_tracker.workout_generator");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = "proto";
    let files = [
        "proto/fitness_tracker/common.proto",
        "proto/fitness_tracker/health_data.proto",
        "proto/fitness_tracker/workout_data.proto",
        "proto/fitness_tracker/workout_generator.proto",
    ];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&files, &[proto_dir])?;

    println!("cargo:rerun-if-changed=proto");
    Ok(())
}

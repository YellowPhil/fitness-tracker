pub trait InferenceProvider {
    type Error: std::error::Error + Send + Sync;

    fn generate_response(&self, prompt: &str) -> Result<String, Self::Error>;
}

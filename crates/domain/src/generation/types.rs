use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use crate::types::{UserId, WorkoutId};

#[derive(Debug, Clone)]
pub struct GenerationJob {
    pub id: Uuid,
    pub user_id: UserId,
    pub date: Date,
    pub status: GenerationJobStatus,
    pub request_fingerprint: String,

    pub request_payload: Value,
    pub workout_id: Option<WorkoutId>,
    pub error: Option<String>,
    pub version: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub queued_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub completed_at: Option<OffsetDateTime>,
    pub failed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationJobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

impl GenerationJobStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub fn parse_api_str(value: &str) -> Option<Self> {
        match value {
            "queued" => Some(Self::Queued),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

impl FromStr for GenerationJobStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_api_str(s).ok_or(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationJobListScope {
    All,
    Active,
}

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Fingerprints {
    #[serde(default)]
    pub must_contain: Vec<String>,
    #[serde(default)]
    pub must_not_contain: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitRequest {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub verification: Vec<String>,
    #[serde(default)]
    pub fingerprints: Option<Fingerprints>,
    #[serde(default)]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    #[serde(skip)]
    pub file: String,
    #[serde(skip)]
    pub state: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct QueueBlocker {
    #[serde(rename = "type")]
    pub kind: String,
    pub ids: Vec<String>,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueReport {
    pub queue_root: String,
    pub pending: usize,
    pub blockers: Vec<QueueBlocker>,
}

pub struct QueueRead {
    pub requests: Vec<CommitRequest>,
    pub blockers: Vec<QueueBlocker>,
}

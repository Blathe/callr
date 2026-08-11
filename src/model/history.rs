use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A point-in-time snapshot of a sent request and its response. Deliberately
/// not linked to the live `RequestFile` by anything but an optional UUID —
/// editing or deleting the source request must never corrupt history.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub request_uuid: Option<Uuid>,
    pub collection_path: Option<String>,
    pub request_name: Option<String>,
    pub method: String,
    pub url: String,
    pub request_headers: Option<String>,
    pub request_body: Option<String>,
    pub auth_type: Option<String>,
    pub status_code: Option<u16>,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub response_size_bytes: Option<i64>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub sent_at: DateTime<Utc>,
}

/// Same shape minus the DB-assigned `id`, used when inserting a new row.
#[derive(Debug, Clone)]
pub struct NewHistoryEntry {
    pub request_uuid: Option<Uuid>,
    pub collection_path: Option<String>,
    pub request_name: Option<String>,
    pub method: String,
    pub url: String,
    pub request_headers: Option<String>,
    pub request_body: Option<String>,
    pub auth_type: Option<String>,
    pub status_code: Option<u16>,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub response_size_bytes: Option<i64>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub sent_at: DateTime<Utc>,
}

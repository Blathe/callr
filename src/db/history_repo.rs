use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use crate::db::connection::SharedConnection;
use crate::db::error::DbError;
use crate::model::history::{HistoryEntry, NewHistoryEntry};

const SELECT_COLUMNS: &str = "id, request_uuid, collection_path, request_name, method, url,
     request_headers, request_body, auth_type, status_code,
     response_headers, response_body, response_size_bytes,
     error_message, duration_ms, sent_at";

pub fn insert(conn: &SharedConnection, entry: &NewHistoryEntry) -> Result<i64, DbError> {
    let conn = conn.lock().expect("history db mutex poisoned");
    conn.execute(
        "INSERT INTO history (
            request_uuid, collection_path, request_name, method, url,
            request_headers, request_body, auth_type, status_code,
            response_headers, response_body, response_size_bytes,
            error_message, duration_ms, sent_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        params![
            entry.request_uuid.map(|u| u.to_string()),
            entry.collection_path,
            entry.request_name,
            entry.method,
            entry.url,
            entry.request_headers,
            entry.request_body,
            entry.auth_type,
            entry.status_code,
            entry.response_headers,
            entry.response_body,
            entry.response_size_bytes,
            entry.error_message,
            entry.duration_ms,
            entry.sent_at.to_rfc3339(),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_recent(conn: &SharedConnection, limit: i64) -> Result<Vec<HistoryEntry>, DbError> {
    let conn = conn.lock().expect("history db mutex poisoned");
    let sql = format!("SELECT {SELECT_COLUMNS} FROM history ORDER BY sent_at DESC LIMIT ?1");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![limit], row_to_entry)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search(conn: &SharedConnection, query: &str, limit: i64) -> Result<Vec<HistoryEntry>, DbError> {
    let conn = conn.lock().expect("history db mutex poisoned");
    let pattern = format!("%{query}%");
    let sql = format!(
        "SELECT {SELECT_COLUMNS} FROM history
         WHERE url LIKE ?1 OR method LIKE ?1 OR IFNULL(request_name, '') LIKE ?1
         ORDER BY sent_at DESC LIMIT ?2"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![pattern, limit], row_to_entry)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<HistoryEntry> {
    let request_uuid: Option<String> = row.get(1)?;
    let sent_at: String = row.get(15)?;
    Ok(HistoryEntry {
        id: row.get(0)?,
        request_uuid: request_uuid.and_then(|s| Uuid::parse_str(&s).ok()),
        collection_path: row.get(2)?,
        request_name: row.get(3)?,
        method: row.get(4)?,
        url: row.get(5)?,
        request_headers: row.get(6)?,
        request_body: row.get(7)?,
        auth_type: row.get(8)?,
        status_code: row.get::<_, Option<i64>>(9)?.map(|v| v as u16),
        response_headers: row.get(10)?,
        response_body: row.get(11)?,
        response_size_bytes: row.get(12)?,
        error_message: row.get(13)?,
        duration_ms: row.get(14)?,
        sent_at: DateTime::parse_from_rfc3339(&sent_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

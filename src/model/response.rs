use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ResponseData {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    /// Lossy UTF-8 decode of the body, for display. Check `is_valid_utf8`
    /// before treating it as authoritative text (a genuinely binary body
    /// decodes to garbage here).
    pub body: String,
    pub is_valid_utf8: bool,
    pub byte_len: usize,
    pub elapsed: Duration,
}

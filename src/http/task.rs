use std::time::Instant;

use tokio::sync::mpsc::UnboundedSender;

use crate::event::AppMsg;
use crate::http::client;
use crate::model::request::RequestFile;
use crate::model::response::ResponseData;

/// Spawns the request send on the tokio runtime so the render/event loop
/// never blocks waiting on the network; the result is reported back over
/// `tx` as an [`AppMsg::HttpResult`], carrying the request that was sent so
/// the caller can build a history snapshot from it.
pub fn spawn_send(http_client: reqwest::Client, tx: UnboundedSender<AppMsg>, request: RequestFile) {
    tokio::spawn(async move {
        let started = Instant::now();
        let result = send(&http_client, &request).await;
        let elapsed = started.elapsed();
        let result = match result {
            Ok(mut data) => {
                data.elapsed = elapsed;
                Ok(data)
            }
            Err(err) => Err(err),
        };
        let _ = tx.send(AppMsg::HttpResult {
            request: Box::new(request),
            result,
        });
    });
}

async fn send(http_client: &reqwest::Client, request: &RequestFile) -> Result<ResponseData, String> {
    let response = client::build_request(http_client, request)
        .send()
        .await
        .map_err(|e| describe_error(&e))?;

    let status = response.status();
    let headers = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect();
    let bytes = response.bytes().await.map_err(|e| describe_error(&e))?;
    let is_valid_utf8 = std::str::from_utf8(&bytes).is_ok();
    let byte_len = bytes.len();
    let body = String::from_utf8_lossy(&bytes).into_owned();

    Ok(ResponseData {
        status: status.as_u16(),
        status_text: status.canonical_reason().unwrap_or_default().to_string(),
        headers,
        body,
        is_valid_utf8,
        byte_len,
        elapsed: std::time::Duration::default(),
    })
}

/// `reqwest::Error`'s `Display` only shows its own top-level message (e.g.
/// "error sending request for url (...)") and drops the actual underlying
/// cause (connection refused, TLS failure, DNS error, ...), which is buried
/// in the `source()` chain instead. Walk it so the user sees the real reason.
fn describe_error(err: &dyn std::error::Error) -> String {
    let mut message = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    #[derive(Debug)]
    struct Chained {
        msg: &'static str,
        source: Option<Box<dyn std::error::Error>>,
    }

    impl fmt::Display for Chained {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.msg)
        }
    }

    impl std::error::Error for Chained {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            self.source.as_deref()
        }
    }

    #[test]
    fn describe_error_walks_the_full_source_chain() {
        let root = Chained {
            msg: "connection refused",
            source: None,
        };
        let mid = Chained {
            msg: "error trying to connect",
            source: Some(Box::new(root)),
        };
        let top = Chained {
            msg: "error sending request for url (https://localhost:7268/api/games/)",
            source: Some(Box::new(mid)),
        };

        let message = describe_error(&top);
        assert_eq!(
            message,
            "error sending request for url (https://localhost:7268/api/games/): error trying to connect: connection refused"
        );
    }

    #[test]
    fn describe_error_handles_no_source() {
        let err = Chained {
            msg: "just a message",
            source: None,
        };
        assert_eq!(describe_error(&err), "just a message");
    }
}

//! A minimal localhost HTTP/1.1 client for calling dependency services.
//!
//! Every srvcs primitive that composes other primitives calls them over HTTP.
//! Rather than take on a full client stack for one concern, this is hand-rolled:
//! open a connection, write a `Connection: close` request, read the response to
//! EOF, split off the body. It only speaks to other srvcs services.

use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// The outcome of calling a dependency's `POST /` evaluate endpoint.
pub enum DepError {
    /// The dependency could not be reached at all (connection refused, etc).
    Unreachable,
}

async fn request(method: &str, url: &str, body: Option<&str>) -> std::io::Result<(u16, String)> {
    let rest = url.strip_prefix("http://").unwrap_or(url);
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };

    let mut stream = TcpStream::connect(authority).await?;
    let body = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\n\
         Host: {authority}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {len}\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        len = body.len(),
    );
    stream.write_all(req.as_bytes()).await?;
    stream.flush().await?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await?;
    let text = String::from_utf8_lossy(&raw).into_owned();

    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((text.as_str(), ""));
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);

    Ok((status, body.to_string()))
}

/// Call a dependency's `POST /` with `{ "value": <value> }` and return its
/// `(status, parsed body)`. Returns `DepError::Unreachable` if the dependency
/// cannot be reached, which the caller surfaces as a degraded `503`.
pub async fn evaluate_dep(base_url: &str, value: &Value) -> Result<(u16, Value), DepError> {
    let body = serde_json::json!({ "value": value }).to_string();
    match request("POST", base_url, Some(&body)).await {
        Ok((status, raw)) => {
            let parsed = serde_json::from_str(&raw).unwrap_or(Value::Null);
            Ok((status, parsed))
        }
        Err(_) => Err(DepError::Unreachable),
    }
}

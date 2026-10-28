//! Loopback-only receiver for the Oyenza Browser Bridge extension.
//!
//! It accepts a tiny JSON notice from a Chrome/Edge extension on this PC.
//! The listener never opens a LAN port and rejects ordinary web-page origins,
//! so a CRM page cannot talk to it directly.

use crate::events::{AppEvent, EventSink};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

pub const PORT: u16 = 32_148;
const MAX_REQUEST: usize = 16 * 1024;

pub struct BrowserBridge;

impl BrowserBridge {
    pub fn start(sink: EventSink) -> io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", PORT))?;
        std::thread::Builder::new()
            .name("poni-browser-bridge".into())
            .spawn(move || accept_loop(listener, sink))
            .map_err(|error| io::Error::other(format!("could not start browser bridge: {error}")))?;
        Ok(Self)
    }
}

fn accept_loop(listener: TcpListener, sink: EventSink) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle(stream, &sink),
            Err(error) => crate::pwarn!("browser bridge: accept failed: {error}"),
        }
    }
}

fn handle(mut stream: TcpStream, sink: &EventSink) {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
    let mut bytes = Vec::with_capacity(2048);
    let mut part = [0u8; 2048];
    while bytes.len() < MAX_REQUEST {
        match stream.read(&mut part) {
            Ok(0) => break,
            Ok(n) => {
                bytes.extend_from_slice(&part[..n]);
                if complete_request(&bytes) { break; }
            }
            Err(_) => return,
        }
    }
    let Some(header_end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") else { return };
    let header = String::from_utf8_lossy(&bytes[..header_end]);
    let origin = header.lines().find_map(|line| line.strip_prefix("Origin: ").or_else(|| line.strip_prefix("origin: "))).unwrap_or("");
    if !extension_origin(origin) {
        reply(&mut stream, "403 Forbidden", "");
        return;
    }
    let method_path = header.lines().next().unwrap_or("");
    if method_path.starts_with("OPTIONS /v1/notice ") {
        reply(&mut stream, "204 No Content", origin);
        return;
    }
    if !method_path.starts_with("POST /v1/notice ") {
        reply(&mut stream, "404 Not Found", origin);
        return;
    }
    let body = &bytes[header_end + 4..];
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        reply(&mut stream, "400 Bad Request", origin);
        return;
    };
    let source = clean(value.get("source").and_then(|v| v.as_str()), 24);
    let title = clean(value.get("title").and_then(|v| v.as_str()), 120);
    let body = clean(value.get("body").and_then(|v| v.as_str()), 220);
    if source.is_empty() || title.is_empty() {
        reply(&mut stream, "400 Bad Request", origin);
        return;
    }
    sink.send(AppEvent::BrowserNotice { source, title, body });
    reply(&mut stream, "204 No Content", origin);
}

fn complete_request(bytes: &[u8]) -> bool {
    let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") else { return false };
    let header = String::from_utf8_lossy(&bytes[..end]);
    let length = header.lines().find_map(|line| line.strip_prefix("Content-Length: ").or_else(|| line.strip_prefix("content-length: "))).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    bytes.len() >= end + 4 + length
}

fn extension_origin(origin: &str) -> bool {
    origin.starts_with("chrome-extension://") || origin.starts_with("moz-extension://")
}

fn clean(value: Option<&str>, limit: usize) -> String {
    value.unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ").chars().take(limit).collect()
}

fn reply(stream: &mut TcpStream, status: &str, origin: &str) {
    let origin = if extension_origin(origin) { origin } else { "null" };
    let response = format!("HTTP/1.1 {status}\r\nAccess-Control-Allow-Origin: {origin}\r\nAccess-Control-Allow-Methods: POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    let _ = stream.write_all(response.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_extension_origins_not_web_pages() {
        assert!(extension_origin("chrome-extension://abcdefghijklmnop"));
        assert!(extension_origin("moz-extension://local-id"));
        assert!(!extension_origin("https://crm-rubin.vercel.app"));
        assert!(!extension_origin(""));
    }

    #[test]
    fn cleans_notice_fields_before_the_ui_sees_them() {
        assert_eq!(clean(Some("  New\n task   from CRM "), 80), "New task from CRM");
        assert_eq!(clean(Some("abcdef"), 3), "abc");
    }
}

use std::ffi::c_int;
use std::sync::OnceLock;
use std::time::Duration;

use curl::easy::{Easy, HttpVersion, List};

const USER_AGENT: &str = concat!("Sunlight/", env!("CARGO_PKG_VERSION"));

pub struct Request {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Default)]
pub struct Response {
    pub status: u32,
    pub message: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub fn send(request: &Request) -> Result<Response, curl::Error> {
    let mut easy = Easy::new();
    easy.url(&request.url)?;
    match request.method.as_str() {
        "GET" | "POST" => {}
        "HEAD" => easy.nobody(true)?,
        method => easy.custom_request(method)?,
    }
    if !matches!(request.method.as_str(), "GET" | "HEAD") {
        easy.post_fields_copy(&request.body)?;
    }
    easy.http_version(HttpVersion::V11)?;
    easy.follow_location(true)?;
    easy.max_redirections(10)?;
    easy.accept_encoding("")?;
    easy.connect_timeout(Duration::from_secs(10))?;
    easy.timeout(Duration::from_secs(30))?;

    let mut list = List::new();
    let has = |header: &str| {
        request
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(header))
    };
    if !has("user-agent") {
        list.append(&format!("User-Agent: {USER_AGENT}"))?;
    }
    if !has("content-type") {
        list.append("Content-Type:")?;
    }
    list.append(&format!("Sunlight-Fingerprint: {}", fingerprint()))?;
    list.append(&format!("Sunlight-User-Identifier: {}", fingerprint()))?;
    for (name, value) in &request.headers {
        list.append(&format!("{name}: {value}"))?;
    }
    easy.http_headers(list)?;

    let mut response = Response::default();
    {
        let mut transfer = easy.transfer();
        transfer.header_function(|line| {
            let line = String::from_utf8_lossy(line);
            let line = line.trim_end();
            if let Some(status) = line.strip_prefix("HTTP/") {
                response.headers.clear();
                response.message = status
                    .splitn(3, ' ')
                    .nth(2)
                    .unwrap_or_default()
                    .to_owned();
            } else if let Some((name, value)) = line.split_once(':') {
                response
                    .headers
                    .push((name.to_owned(), value.trim().to_owned()));
            }
            true
        })?;
        transfer.write_function(|data| {
            response.body.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }
    response.status = easy.response_code()?;
    Ok(response)
}

unsafe extern "C" {
    fn gethostuuid(id: *mut [u8; 16], wait: *const [i64; 2]) -> c_int;
}

fn fingerprint() -> &'static str {
    static FINGERPRINT: OnceLock<String> = OnceLock::new();
    FINGERPRINT.get_or_init(|| {
        let mut id = [0; 16];
        unsafe { gethostuuid(&mut id, &[0; 2]) };
        id.iter().map(|byte| format!("{byte:02x}")).collect()
    })
}

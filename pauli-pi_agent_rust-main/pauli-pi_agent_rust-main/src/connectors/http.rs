//! HTTP/network connector with policy-gated access.
//!
//! Provides basic fetch (GET/POST) with:
//! - Host allowlist/denylist
//! - TLS required by default
//! - Request timeouts and size limits
//! - Structured logging for audit trail

use super::{
    Connector, HostCallErrorCode, HostCallPayload, HostResultPayload, host_result_err,
    host_result_err_with_details, host_result_ok,
};
use crate::error::Result;
use crate::http::client::Client;
use asupersync::time::{timeout, wall_now};
use async_trait::async_trait;
use futures::Stream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::pin::Pin;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Validation error with error code and message.
type ValidationError = (HostCallErrorCode, String);

/// Configuration for the HTTP connector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConnectorConfig {
    /// Host patterns to allow (glob-style: "*.example.com", "api.github.com")
    #[serde(default)]
    pub allowlist: Vec<String>,

    /// Host patterns to deny (takes precedence over allowlist)
    #[serde(default)]
    pub denylist: Vec<String>,

    /// When true, an empty allowlist denies all outbound hosts.
    #[serde(default)]
    pub enforce_allowlist: bool,

    /// Require TLS for all requests (default: true)
    #[serde(default = "default_require_tls")]
    pub require_tls: bool,

    /// Maximum request body size in bytes (default: 10MB)
    #[serde(default = "default_max_request_bytes")]
    pub max_request_bytes: usize,

    /// Maximum response body size in bytes (default: 50MB)
    #[serde(default = "default_max_response_bytes")]
    pub max_response_bytes: usize,

    /// Default timeout in milliseconds (default: 30000)
    #[serde(default = "default_timeout_ms")]
    pub default_timeout_ms: u64,
}

const fn default_require_tls() -> bool {
    true
}

const fn default_max_request_bytes() -> usize {
    10 * 1024 * 1024 // 10MB
}

const fn default_max_response_bytes() -> usize {
    50 * 1024 * 1024 // 50MB
}

const fn default_timeout_ms() -> u64 {
    30_000 // 30 seconds
}

impl Default for HttpConnectorConfig {
    fn default() -> Self {
        Self {
            allowlist: Vec::new(),
            denylist: Vec::new(),
            enforce_allowlist: false,
            require_tls: default_require_tls(),
            max_request_bytes: default_max_request_bytes(),
            max_response_bytes: default_max_response_bytes(),
            default_timeout_ms: default_timeout_ms(),
        }
    }
}

/// HTTP request parameters from hostcall.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// The URL to fetch
    pub url: String,

    /// HTTP method (GET, POST)
    #[serde(default = "default_method")]
    pub method: String,

    /// Request headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Request body (for POST)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Request body as bytes (base64-encoded)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_bytes: Option<String>,

    /// Override timeout in milliseconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

fn default_method() -> String {
    "GET".to_string()
}

/// HTTP response returned to extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Response body as string (if text)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Response body as bytes (base64-encoded, if binary)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_bytes: Option<String>,

    /// Response body size in bytes
    pub size_bytes: usize,

    /// Request duration in milliseconds
    pub duration_ms: u64,
}

/// Streaming HTTP response returned to the host dispatcher.
///
/// This intentionally returns only the response head plus a byte stream. The caller
/// is responsible for chunking/decoding (UTF-8/base64), SSE parsing, idle timeouts,
/// and delivering `StreamChunk` outcomes to the extension runtime.
pub struct StreamingHttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub stream: Pin<Box<dyn Stream<Item = std::io::Result<Vec<u8>>> + Send>>,
}

/// HTTP connector for extension hostcalls.
pub struct HttpConnector {
    config: HttpConnectorConfig,
    client: Client,
}

impl HttpConnector {
    /// Create a new HTTP connector with the given configuration.
    #[must_use]
    pub fn new(config: HttpConnectorConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Create a new HTTP connector with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(HttpConnectorConfig::default())
    }

    /// Validate a URL against the policy.
    fn validate_url(&self, url: &str) -> std::result::Result<(), ValidationError> {
        // Parse URL to extract host
        let parsed = url::Url::parse(url).map_err(|e| {
            (
                HostCallErrorCode::InvalidRequest,
                format!("Invalid URL: {e}"),
            )
        })?;

        // HTTP/HTTPS only
        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return Err((
                HostCallErrorCode::InvalidRequest,
                format!("Unsupported URL scheme: '{scheme}'"),
            ));
        }

        // Check scheme (TLS requirement)
        if self.config.require_tls && scheme == "http" {
            return Err((
                HostCallErrorCode::Denied,
                format!("TLS required: URL scheme must be 'https', got '{scheme}'"),
            ));
        }

        // Extract host
        let host = parsed.host_str().ok_or_else(|| {
            (
                HostCallErrorCode::InvalidRequest,
                "URL missing host".to_string(),
            )
        })?;

        // Check denylist first (takes precedence)
        if Self::matches_pattern_list(host, &self.config.denylist) {
            return Err((
                HostCallErrorCode::Denied,
                format!("Host '{host}' is in denylist"),
            ));
        }

        // Check allowlist. In strict mode, empty allowlist means deny-all.
        let requires_allowlist = self.config.enforce_allowlist || !self.config.allowlist.is_empty();
        if requires_allowlist && !Self::matches_pattern_list(host, &self.config.allowlist) {
            return Err((
                HostCallErrorCode::Denied,
                format!(
                    "Host '{host}' is not in allowlist; declare capability_manifest scope.hosts for http capability"
                ),
            ));
        }

        Ok(())
    }

    /// Check if a host matches any pattern in the list.
    fn matches_pattern_list(host: &str, patterns: &[String]) -> bool {
        let host_lower = host.to_ascii_lowercase();
        patterns.iter().any(|pattern| {
            let pattern_lower = pattern.to_ascii_lowercase();
            pattern_lower.strip_prefix("*.").map_or_else(
                || host_lower == pattern_lower,
                |domain| {
                    // Wildcard subdomain match: "*.example.com" matches "api.example.com"
                    let suffix = pattern_lower.strip_prefix('*').unwrap_or(""); // ".example.com"
                    host_lower.ends_with(suffix) || host_lower == domain
                },
            )
        })
    }

    /// Parse and validate the HTTP request from hostcall params.
    fn parse_request(&self, params: &Value) -> std::result::Result<HttpRequest, ValidationError> {
        let mut request: HttpRequest = serde_json::from_value(params.clone()).map_err(|e| {
            (
                HostCallErrorCode::InvalidRequest,
                format!("Invalid HTTP request params: {e}"),
            )
        })?;

        if request.body.is_some() && request.body_bytes.is_some() {
            return Err((
                HostCallErrorCode::InvalidRequest,
                "Request must specify either 'body' or 'body_bytes', not both".to_string(),
            ));
        }

        // Validate method (connector supports GET/POST only)
        let method_upper = request.method.to_ascii_uppercase();
        if !matches!(method_upper.as_str(), "GET" | "POST") {
            return Err((
                HostCallErrorCode::InvalidRequest,
                format!(
                    "Invalid HTTP method: '{}'. Supported methods: GET, POST.",
                    request.method
                ),
            ));
        }

        // Treat 0 as unset/absent to match core hostcall timeout semantics.
        request.timeout_ms = request.timeout_ms.filter(|ms| *ms > 0);

        // Validate body size
        let body_size = request
            .body
            .as_ref()
            .map(String::len)
            .or_else(|| {
                request.body_bytes.as_ref().map(|b| b.len() * 3 / 4) // base64 decode estimate
            })
            .unwrap_or(0);

        if body_size > self.config.max_request_bytes {
            return Err((
                HostCallErrorCode::InvalidRequest,
                format!(
                    "Request body too large: {} bytes (max: {} bytes)",
                    body_size, self.config.max_request_bytes
                ),
            ));
        }

        if method_upper == "GET" && (request.body.is_some() || request.body_bytes.is_some()) {
            return Err((
                HostCallErrorCode::InvalidRequest,
                "GET requests cannot include a body".to_string(),
            ));
        }

        Ok(request)
    }

    /// Execute the HTTP request.
    async fn execute_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let start = Instant::now();

        // Build request
        let method_upper = request.method.to_ascii_uppercase();
        let mut builder = match method_upper.as_str() {
            "GET" => self.client.get(&request.url),
            "POST" => self.client.post(&request.url),
            _ => {
                return Err(crate::error::Error::validation(format!(
                    "Invalid HTTP method: '{}'. Supported methods: GET, POST.",
                    request.method
                )));
            }
        };

        // Add headers
        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }

        // Add body if present
        if let Some(body) = &request.body {
            builder = builder.body(body.as_bytes().to_vec());
        } else if let Some(body_bytes) = &request.body_bytes {
            use base64::Engine;
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(body_bytes)
                .map_err(|e| {
                    crate::error::Error::validation(format!("Invalid base64 body: {e}"))
                })?;
            builder = builder.body(decoded);
        }

        // Send request
        let response = builder
            .send()
            .await
            .map_err(|e| crate::error::Error::extension(format!("HTTP request failed: {e}")))?;

        // Read response body with size limit
        let status = response.status();
        let response_headers: Vec<(String, String)> = response.headers().to_vec();

        let mut body_bytes_vec = Vec::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk: Vec<u8> = chunk_result
                .map_err(|e| crate::error::Error::extension(format!("Read error: {e}")))?;
            if body_bytes_vec.len() + chunk.len() > self.config.max_response_bytes {
                return Err(crate::error::Error::extension(format!(
                    "Response body too large (max: {} bytes)",
                    self.config.max_response_bytes
                )));
            }
            body_bytes_vec.extend_from_slice(&chunk);
        }

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let size_bytes = body_bytes_vec.len();

        // Convert headers to HashMap
        let mut headers_map = HashMap::new();
        for (key, value) in response_headers {
            headers_map.insert(key, value);
        }

        // Try to decode body as UTF-8, fall back to base64.
        let (body, body_bytes_b64) = String::from_utf8(body_bytes_vec).map_or_else(
            |err| {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(err.into_bytes());
                (None, Some(encoded))
            },
            |s| (Some(s), None),
        );

        Ok(HttpResponse {
            status,
            headers: headers_map,
            body,
            body_bytes: body_bytes_b64,
            size_bytes,
            duration_ms,
        })
    }

    fn request_details(request: &HttpRequest, timeout_ms: u64) -> Value {
        json!({
            "url": request.url,
            "method": request.method,
            "timeout_ms": timeout_ms,
        })
    }

    fn redact_url_for_log(url: &str) -> String {
        url::Url::parse(url).map_or_else(
            |_| url.split(['?', '#']).next().unwrap_or(url).to_string(),
            |mut parsed| {
                parsed.set_query(None);
                parsed.set_fragment(None);
                let _ = parsed.set_username("");
                let _ = parsed.set_password(None);
                parsed.to_string()
            },
        )
    }

    async fn dispatch_request(&self, call_id: &str, request: HttpRequest) -> HostResultPayload {
        let log_url = Self::redact_url_for_log(&request.url);
        // Validate URL against policy
        if let Err((code, message)) = self.validate_url(&request.url) {
            info!(
                call_id = %call_id,
                url = %log_url,
                error = %message,
                "HTTP connector: policy denied"
            );
            return host_result_err(call_id, code, message, None);
        }

        // Log request
        debug!(
            call_id = %call_id,
            url = %log_url,
            method = %request.method,
            "HTTP connector: executing request"
        );

        // Execute request with timeout for the full request/response read.
        let timeout_ms = request.timeout_ms.unwrap_or(self.config.default_timeout_ms);
        let start = Instant::now();
        let result = if timeout_ms == 0 {
            Ok(self.execute_request(&request).await)
        } else {
            timeout(
                wall_now(),
                Duration::from_millis(timeout_ms),
                Box::pin(self.execute_request(&request)),
            )
            .await
        };

        match result {
            Ok(Ok(response)) => {
                info!(
                    call_id = %call_id,
                    url = %log_url,
                    status = %response.status,
                    size_bytes = %response.size_bytes,
                    duration_ms = %response.duration_ms,
                    "HTTP connector: request completed"
                );

                let output = serde_json::to_value(&response)
                    .unwrap_or_else(|_| json!({"error": "serialization_failed"}));

                host_result_ok(call_id, output)
            }
            Ok(Err(e)) => {
                if timeout_ms > 0 && start.elapsed() >= Duration::from_millis(timeout_ms) {
                    let message = format!("Request timeout after {timeout_ms}ms");
                    warn!(
                        call_id = %call_id,
                        url = %log_url,
                        error = %message,
                        "HTTP connector: request timed out"
                    );

                    return host_result_err_with_details(
                        call_id,
                        HostCallErrorCode::Timeout,
                        &message,
                        Self::request_details(&request, timeout_ms),
                        Some(true),
                    );
                }

                let message = e.to_string();
                let code = match e {
                    crate::error::Error::Validation(_) => HostCallErrorCode::InvalidRequest,
                    _ => HostCallErrorCode::Io,
                };

                warn!(
                    call_id = %call_id,
                    url = %log_url,
                    error = %message,
                    "HTTP connector: request failed"
                );

                host_result_err_with_details(
                    call_id,
                    code,
                    &message,
                    Self::request_details(&request, timeout_ms),
                    Some(false),
                )
            }
            Err(_) => {
                let message = format!("Request timeout after {timeout_ms}ms");
                warn!(
                    call_id = %call_id,
                    url = %log_url,
                    error = %message,
                    "HTTP connector: request timed out"
                );

                host_result_err_with_details(
                    call_id,
                    HostCallErrorCode::Timeout,
                    &message,
                    Self::request_details(&request, timeout_ms),
                    Some(true),
                )
            }
        }
    }

    /// Dispatch an HTTP request but return a streaming response body instead of buffering it.
    ///
    /// Errors are returned as a `HostResultPayload` (taxonomy-correct) so the caller can
    /// convert into `HostcallOutcome::Error` deterministically.
    pub async fn dispatch_streaming(
        &self,
        call: &HostCallPayload,
    ) -> std::result::Result<StreamingHttpResponse, HostResultPayload> {
        let call_id = &call.call_id;
        let method = call.method.to_ascii_lowercase();

        if method != "http" {
            warn!(
                call_id = %call_id,
                method = %method,
                "HTTP connector: unsupported method (streaming)"
            );
            return Err(host_result_err(
                call_id,
                HostCallErrorCode::InvalidRequest,
                format!("Unsupported HTTP connector method: '{method}'. Use 'http'."),
                None,
            ));
        }

        let mut request = match self.parse_request(&call.params) {
            Ok(req) => req,
            Err((code, message)) => {
                warn!(
                    call_id = %call_id,
                    error = %message,
                    "HTTP connector: invalid request (streaming)"
                );
                return Err(host_result_err(call_id, code, message, None));
            }
        };

        // Prefer explicit per-request timeout in params, otherwise fall back to host_call.timeout_ms.
        if request.timeout_ms.is_none() {
            request.timeout_ms = call.timeout_ms.filter(|ms| *ms > 0);
        }

        let log_url = Self::redact_url_for_log(&request.url);
        if let Err((code, message)) = self.validate_url(&request.url) {
            info!(
                call_id = %call_id,
                url = %log_url,
                error = %message,
                "HTTP connector: policy denied (streaming)"
            );
            return Err(host_result_err(call_id, code, message, None));
        }

        debug!(
            call_id = %call_id,
            url = %log_url,
            method = %request.method,
            "HTTP connector: executing request (streaming)"
        );

        let timeout_ms = request.timeout_ms.unwrap_or(self.config.default_timeout_ms);
        let (response, duration_ms) = match self
            .dispatch_request_streaming_head(call_id, &request, timeout_ms, &log_url)
            .await
        {
            Ok(res) => res,
            Err(payload) => return Err(payload),
        };

        let status = response.status();
        let response_headers: Vec<(String, String)> = response.headers().to_vec();

        let mut headers_map = HashMap::new();
        for (key, value) in response_headers {
            headers_map.insert(key, value);
        }

        info!(
            call_id = %call_id,
            url = %log_url,
            status = status,
            duration_ms = duration_ms,
            "HTTP connector: streaming response head received"
        );

        Ok(StreamingHttpResponse {
            status,
            headers: headers_map,
            stream: response.bytes_stream(),
        })
    }

    #[allow(clippy::future_not_send)]
    async fn dispatch_request_streaming_head(
        &self,
        call_id: &str,
        request: &HttpRequest,
        timeout_ms: u64,
        log_url: &str,
    ) -> std::result::Result<(crate::http::client::Response, u64), HostResultPayload> {
        let start = Instant::now();
        let builder = match self.build_streaming_request_builder(call_id, request, timeout_ms) {
            Ok(builder) => builder,
            Err(payload) => return Err(*payload),
        };
        let send_fut = builder.send();
        let result = if timeout_ms == 0 {
            Ok(send_fut.await)
        } else {
            timeout(
                wall_now(),
                Duration::from_millis(timeout_ms),
                Box::pin(send_fut),
            )
            .await
        };

        match result {
            Ok(Ok(response)) => {
                let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
                Ok((response, duration_ms))
            }
            Ok(Err(err)) => {
                if timeout_ms > 0 && start.elapsed() >= Duration::from_millis(timeout_ms) {
                    let message = format!("Request timeout after {timeout_ms}ms");
                    warn!(
                        call_id = %call_id,
                        url = %log_url,
                        error = %message,
                        "HTTP connector: request timed out (streaming)"
                    );

                    return Err(host_result_err_with_details(
                        call_id,
                        HostCallErrorCode::Timeout,
                        &message,
                        Self::request_details(request, timeout_ms),
                        Some(true),
                    ));
                }

                let message = err.to_string();
                let code = match err {
                    crate::error::Error::Validation(_) => HostCallErrorCode::InvalidRequest,
                    _ => HostCallErrorCode::Io,
                };

                warn!(
                    call_id = %call_id,
                    url = %log_url,
                    error = %message,
                    "HTTP connector: request failed (streaming)"
                );

                Err(host_result_err_with_details(
                    call_id,
                    code,
                    &message,
                    Self::request_details(request, timeout_ms),
                    Some(false),
                ))
            }
            Err(_) => {
                let message = format!("Request timeout after {timeout_ms}ms");
                warn!(
                    call_id = %call_id,
                    url = %log_url,
                    error = %message,
                    "HTTP connector: request timed out (streaming)"
                );

                Err(host_result_err_with_details(
                    call_id,
                    HostCallErrorCode::Timeout,
                    &message,
                    Self::request_details(request, timeout_ms),
                    Some(true),
                ))
            }
        }
    }

    fn build_streaming_request_builder<'a>(
        &'a self,
        call_id: &str,
        request: &HttpRequest,
        timeout_ms: u64,
    ) -> std::result::Result<crate::http::client::RequestBuilder<'a>, Box<HostResultPayload>> {
        let method_upper = request.method.to_ascii_uppercase();
        let mut builder = match method_upper.as_str() {
            "GET" => self.client.get(&request.url),
            "POST" => self.client.post(&request.url),
            _ => {
                return Err(Box::new(host_result_err_with_details(
                    call_id,
                    HostCallErrorCode::InvalidRequest,
                    format!(
                        "Invalid HTTP method: '{}'. Supported methods: GET, POST.",
                        request.method
                    ),
                    Self::request_details(request, timeout_ms),
                    Some(false),
                )));
            }
        };

        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }

        if let Some(body) = &request.body {
            builder = builder.body(body.as_bytes().to_vec());
        } else if let Some(body_bytes) = &request.body_bytes {
            use base64::Engine;
            let decoded = match base64::engine::general_purpose::STANDARD.decode(body_bytes) {
                Ok(decoded) => decoded,
                Err(err) => {
                    return Err(Box::new(host_result_err_with_details(
                        call_id,
                        HostCallErrorCode::InvalidRequest,
                        format!("Invalid base64 body: {err}"),
                        Self::request_details(request, timeout_ms),
                        Some(false),
                    )));
                }
            };
            builder = builder.body(decoded);
        }

        Ok(builder)
    }
}

#[async_trait]
impl Connector for HttpConnector {
    fn capability(&self) -> &'static str {
        "http"
    }

    #[allow(clippy::too_many_lines)]
    async fn dispatch(&self, call: &HostCallPayload) -> Result<HostResultPayload> {
        let call_id = &call.call_id;
        let method = call.method.to_ascii_lowercase();

        // Protocol expects connector method name "http".
        if method != "http" {
            warn!(
                call_id = %call_id,
                method = %method,
                "HTTP connector: unsupported method"
            );
            return Ok(host_result_err(
                call_id,
                HostCallErrorCode::InvalidRequest,
                format!("Unsupported HTTP connector method: '{method}'. Use 'http'."),
                None,
            ));
        }

        // Parse request
        let mut request = match self.parse_request(&call.params) {
            Ok(req) => req,
            Err((code, message)) => {
                warn!(
                    call_id = %call_id,
                    error = %message,
                    "HTTP connector: invalid request"
                );
                return Ok(host_result_err(call_id, code, message, None));
            }
        };

        // Prefer explicit per-request timeout in params, otherwise fall back to host_call.timeout_ms.
        if request.timeout_ms.is_none() {
            request.timeout_ms = call.timeout_ms.filter(|ms| *ms > 0);
        }

        Ok(self.dispatch_request(call_id, request).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    fn run_async<T, Fut>(future: Fut) -> T
    where
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
            .build()
            .expect("build asupersync runtime");
        let join = runtime.handle().spawn(future);
        runtime.block_on(join)
    }

    #[test]
    fn test_default_config() {
        let config = HttpConnectorConfig::default();
        assert!(config.require_tls);
        assert_eq!(config.max_request_bytes, 10 * 1024 * 1024);
        assert_eq!(config.max_response_bytes, 50 * 1024 * 1024);
        assert_eq!(config.default_timeout_ms, 30_000);
        assert!(config.allowlist.is_empty());
        assert!(config.denylist.is_empty());
        assert!(!config.enforce_allowlist);
    }

    #[test]
    fn test_url_validation_tls_required() {
        let connector = HttpConnector::new(HttpConnectorConfig {
            require_tls: true,
            ..Default::default()
        });

        // HTTPS should pass
        assert!(connector.validate_url("https://example.com").is_ok());

        // HTTP should fail when TLS required
        let result = connector.validate_url("http://example.com");
        assert!(result.is_err());
        let (code, _) = result.unwrap_err();
        assert_eq!(code, HostCallErrorCode::Denied);
    }

    #[test]
    fn test_url_validation_tls_not_required() {
        let connector = HttpConnector::new(HttpConnectorConfig {
            require_tls: false,
            ..Default::default()
        });

        // Both should pass
        assert!(connector.validate_url("https://example.com").is_ok());
        assert!(connector.validate_url("http://example.com").is_ok());
    }

    #[test]
    fn test_url_validation_allowlist() {
        let connector = HttpConnector::new(HttpConnectorConfig {
            require_tls: false,
            allowlist: vec!["api.example.com".to_string(), "*.github.com".to_string()],
            ..Default::default()
        });

        // Exact match should pass
        assert!(
            connector
                .validate_url("http://api.example.com/path")
                .is_ok()
        );

        // Wildcard match should pass
        assert!(connector.validate_url("http://api.github.com/path").is_ok());
        assert!(connector.validate_url("http://raw.github.com/path").is_ok());

        // Non-matching should fail
        let result = connector.validate_url("http://other.com/path");
        assert!(result.is_err());
        let (code, _) = result.unwrap_err();
        assert_eq!(code, HostCallErrorCode::Denied);
    }

    #[test]
    fn test_url_validation_enforced_allowlist_denies_when_empty() {
        let connector = HttpConnector::new(HttpConnectorConfig {
            require_tls: false,
            enforce_allowlist: true,
            ..Default::default()
        });

        let result = connector.validate_url("http://example.com/path");
        assert!(result.is_err());
        let (code, msg) = result.unwrap_err();
        assert_eq!(code, HostCallErrorCode::Denied);
        assert!(msg.contains("allowlist"), "{msg}");
    }

    #[test]
    fn test_url_validation_denylist() {
        let connector = HttpConnector::new(HttpConnectorConfig {
            require_tls: false,
            denylist: vec!["evil.com".to_string(), "*.malware.net".to_string()],
            ..Default::default()
        });

        // Non-denied should pass
        assert!(connector.validate_url("http://example.com/path").is_ok());

        // Exact deny match should fail
        let result = connector.validate_url("http://evil.com/path");
        assert!(result.is_err());
        let (code, _) = result.unwrap_err();
        assert_eq!(code, HostCallErrorCode::Denied);

        // Wildcard deny match should fail
        let result = connector.validate_url("http://api.malware.net/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_url_validation_denylist_precedence() {
        let connector = HttpConnector::new(HttpConnectorConfig {
            require_tls: false,
            allowlist: vec!["*.example.com".to_string()],
            denylist: vec!["evil.example.com".to_string()],
            ..Default::default()
        });

        // Allowed subdomain should pass
        assert!(
            connector
                .validate_url("http://api.example.com/path")
                .is_ok()
        );

        // Denied subdomain should fail (denylist takes precedence)
        let result = connector.validate_url("http://evil.example.com/path");
        assert!(result.is_err());
        let (code, _) = result.unwrap_err();
        assert_eq!(code, HostCallErrorCode::Denied);
    }

    #[test]
    fn test_pattern_matching() {
        let wildcard_patterns = vec!["*.example.com".to_string()];

        // Test wildcard patterns
        assert!(HttpConnector::matches_pattern_list(
            "api.example.com",
            &wildcard_patterns
        ));
        assert!(HttpConnector::matches_pattern_list(
            "sub.api.example.com",
            &wildcard_patterns
        ));
        assert!(HttpConnector::matches_pattern_list(
            "example.com",
            &wildcard_patterns
        ));

        // Test exact patterns
        let exact_patterns = vec!["example.com".to_string()];
        assert!(HttpConnector::matches_pattern_list(
            "example.com",
            &exact_patterns
        ));
        assert!(!HttpConnector::matches_pattern_list(
            "api.example.com",
            &exact_patterns
        ));

        // Test case insensitivity
        assert!(HttpConnector::matches_pattern_list(
            "API.Example.COM",
            &wildcard_patterns
        ));
    }

    #[test]
    fn test_parse_request_valid() {
        let connector = HttpConnector::with_defaults();

        let params = json!({
            "url": "https://api.example.com/data",
            "method": "POST",
            "headers": {"Content-Type": "application/json"},
            "body": "{\"key\": \"value\"}"
        });

        let request = connector.parse_request(&params).unwrap();
        assert_eq!(request.url, "https://api.example.com/data");
        assert_eq!(request.method, "POST");
        assert_eq!(
            request.headers.get("Content-Type").unwrap(),
            "application/json"
        );
        assert_eq!(request.body.as_ref().unwrap(), "{\"key\": \"value\"}");
    }

    #[test]
    fn test_parse_request_invalid_method() {
        let connector = HttpConnector::with_defaults();

        let params = json!({
            "url": "https://api.example.com/data",
            "method": "INVALID"
        });

        let result = connector.parse_request(&params);
        assert!(result.is_err());
        let (code, _) = result.unwrap_err();
        assert_eq!(code, HostCallErrorCode::InvalidRequest);
    }

    #[test]
    fn test_parse_request_body_too_large() {
        let connector = HttpConnector::new(HttpConnectorConfig {
            max_request_bytes: 100,
            ..Default::default()
        });

        let large_body = "x".repeat(200);
        let params = json!({
            "url": "https://api.example.com/data",
            "method": "POST",
            "body": large_body
        });

        let result = connector.parse_request(&params);
        assert!(result.is_err());
        let (code, _) = result.unwrap_err();
        assert_eq!(code, HostCallErrorCode::InvalidRequest);
    }

    #[test]
    fn test_parse_request_rejects_both_body_and_body_bytes() {
        let connector = HttpConnector::with_defaults();

        let params = json!({
            "url": "https://api.example.com/data",
            "method": "POST",
            "body": "{\"key\": \"value\"}",
            "body_bytes": "eyJrZXkiOiAidmFsdWUifQ=="
        });

        let result = connector.parse_request(&params);
        assert!(result.is_err());
        let (code, message) = result.unwrap_err();
        assert_eq!(code, HostCallErrorCode::InvalidRequest);
        assert!(
            message.contains("not both"),
            "expected ambiguity error, got: {message}"
        );
    }

    #[test]
    fn test_config_serialization() {
        let config = HttpConnectorConfig {
            allowlist: vec!["*.example.com".to_string()],
            denylist: vec!["evil.com".to_string()],
            enforce_allowlist: true,
            require_tls: true,
            max_request_bytes: 1024,
            max_response_bytes: 2048,
            default_timeout_ms: 5000,
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: HttpConnectorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.allowlist, config.allowlist);
        assert_eq!(parsed.denylist, config.denylist);
        assert_eq!(parsed.enforce_allowlist, config.enforce_allowlist);
        assert_eq!(parsed.require_tls, config.require_tls);
        assert_eq!(parsed.max_request_bytes, config.max_request_bytes);
        assert_eq!(parsed.max_response_bytes, config.max_response_bytes);
        assert_eq!(parsed.default_timeout_ms, config.default_timeout_ms);
    }

    #[test]
    fn test_dispatch_denied_host_returns_deterministic_error() {
        let connector = HttpConnector::new(HttpConnectorConfig {
            require_tls: false,
            allowlist: vec!["allowed.example".to_string()],
            ..Default::default()
        });

        let call = HostCallPayload {
            call_id: "call-1".to_string(),
            capability: "http".to_string(),
            method: "http".to_string(),
            params: json!({
                "url": "http://denied.example/test",
                "method": "GET",
            }),
            timeout_ms: None,
            cancel_token: None,
            context: None,
        };

        let result = run_async(async move { connector.dispatch(&call).await.unwrap() });
        assert!(result.is_error);
        let error = result.error.expect("error payload");
        assert_eq!(error.code, HostCallErrorCode::Denied);
    }

    #[test]
    fn test_dispatch_timeout_returns_timeout_error_code() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let addr = listener.local_addr().expect("listener addr");

        let (ready_tx, ready_rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let join = thread::spawn(move || {
            let _ = ready_tx.send(());
            let (_stream, _peer) = listener.accept().expect("accept");
            let _ = shutdown_rx.recv_timeout(std::time::Duration::from_millis(500));
        });
        let _ = ready_rx.recv();

        let connector = HttpConnector::new(HttpConnectorConfig {
            require_tls: false,
            default_timeout_ms: 100,
            ..Default::default()
        });

        let call = HostCallPayload {
            call_id: "call-1".to_string(),
            capability: "http".to_string(),
            method: "http".to_string(),
            params: json!({
                "url": format!("http://{addr}/"),
                "method": "GET",
                "timeout_ms": 100,
            }),
            timeout_ms: None,
            cancel_token: None,
            context: None,
        };

        let result = run_async(async move { connector.dispatch(&call).await.unwrap() });
        assert!(result.is_error);
        let error = result.error.expect("error payload");
        assert_eq!(error.code, HostCallErrorCode::Timeout);
        assert_eq!(error.retryable, Some(true));

        let _ = shutdown_tx.send(());
        let _ = join.join();
    }

    #[test]
    fn test_dispatch_uses_call_timeout_ms_when_request_timeout_absent() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let addr = listener.local_addr().expect("listener addr");

        let (ready_tx, ready_rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let join = thread::spawn(move || {
            let _ = ready_tx.send(());
            let (_stream, _peer) = listener.accept().expect("accept");
            let _ = shutdown_rx.recv_timeout(std::time::Duration::from_millis(500));
        });
        let _ = ready_rx.recv();

        // Default timeout is large; call.timeout_ms should take precedence when params omit it.
        let connector = HttpConnector::new(HttpConnectorConfig {
            require_tls: false,
            default_timeout_ms: 5000,
            ..Default::default()
        });

        let call = HostCallPayload {
            call_id: "call-1".to_string(),
            capability: "http".to_string(),
            method: "http".to_string(),
            params: json!({
                "url": format!("http://{addr}/"),
                "method": "GET",
            }),
            timeout_ms: Some(100),
            cancel_token: None,
            context: None,
        };

        let result = run_async(async move { connector.dispatch(&call).await.unwrap() });
        assert!(result.is_error);
        let error = result.error.expect("error payload");
        assert!(
            error.code == HostCallErrorCode::Timeout,
            "expected timeout, got {:?} (details={:?})",
            error.code,
            error.details
        );

        let _ = shutdown_tx.send(());
        let _ = join.join();
    }

    #[test]
    #[cfg(unix)]
    fn test_dispatch_treats_zero_timeout_as_unset() {
        use std::io::Write;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let addr = listener.local_addr().expect("listener addr");

        let join = thread::spawn(move || {
            let (mut stream, _peer) = listener.accept().expect("accept");
            let body = "hello";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        });

        let connector = HttpConnector::new(HttpConnectorConfig {
            require_tls: false,
            default_timeout_ms: 5000,
            ..Default::default()
        });

        let call = HostCallPayload {
            call_id: "call-1".to_string(),
            capability: "http".to_string(),
            method: "http".to_string(),
            params: json!({
                "url": format!("http://{addr}/"),
                "method": "GET",
                "timeout_ms": 0,
            }),
            timeout_ms: None,
            cancel_token: None,
            context: None,
        };

        let result = run_async(async move { connector.dispatch(&call).await.unwrap() });
        assert!(!result.is_error);
        assert_eq!(
            result.output.get("status").and_then(Value::as_u64),
            Some(200)
        );
        assert_eq!(
            result.output.get("body").and_then(Value::as_str),
            Some("hello")
        );

        let _ = join.join();
    }

    #[test]
    #[cfg(unix)]
    fn test_dispatch_streaming_returns_status_headers_and_body_stream() {
        use futures::StreamExt as _;
        use std::io::Write;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let addr = listener.local_addr().expect("listener addr");

        let join = thread::spawn(move || {
            let (mut stream, _peer) = listener.accept().expect("accept");
            let body = "hello-stream";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        });

        let connector = HttpConnector::new(HttpConnectorConfig {
            require_tls: false,
            default_timeout_ms: 5000,
            ..Default::default()
        });

        let call = HostCallPayload {
            call_id: "call-1".to_string(),
            capability: "http".to_string(),
            method: "http".to_string(),
            params: json!({
                "url": format!("http://{addr}/"),
                "method": "GET",
                "timeout_ms": 1000,
            }),
            timeout_ms: None,
            cancel_token: None,
            context: None,
        };

        let (status, headers, body) = run_async(async move {
            let response = connector
                .dispatch_streaming(&call)
                .await
                .expect("dispatch_streaming ok");

            let mut bytes = Vec::new();
            let mut stream = response.stream;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.expect("stream chunk");
                bytes.extend_from_slice(&chunk);
            }

            (response.status, response.headers, bytes)
        });

        assert_eq!(status, 200);
        assert_eq!(
            headers
                .get("Content-Type")
                .or_else(|| headers.get("content-type"))
                .map(String::as_str),
            Some("text/plain")
        );
        assert_eq!(String::from_utf8_lossy(&body), "hello-stream");

        let _ = join.join();
    }

    #[test]
    fn http_connector_redact_url_for_log_strips_sensitive_parts() {
        let redacted =
            HttpConnector::redact_url_for_log("http://user:pass@denied.example/test?q=hello#frag");
        assert!(redacted.contains("http://denied.example/test"));
        assert!(!redacted.contains("q=hello"));
        assert!(!redacted.contains("#frag"));
        assert!(!redacted.contains("user"));
        assert!(!redacted.contains("pass"));
    }

    #[test]
    fn http_connector_redact_url_for_log_falls_back_for_invalid_urls() {
        let redacted = HttpConnector::redact_url_for_log("not a url?q=hello#frag");
        assert_eq!(redacted, "not a url");
    }
}

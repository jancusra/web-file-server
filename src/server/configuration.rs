//! Server configuration: available files to serving, defined MIME types and some related methods

use std::{collections::HashMap, ffi::OsStr, path::Path, time::Duration};

/// A file that is publicly served, with its path relative to the web root and metadata
#[derive(Debug, Clone)]
pub struct ServedFile {
    /// Path relative to [`ServerConfig::web_path`], resolved at request time
    pub rel_path: String,
    pub content_type: String,
    pub cache: bool,
}

/// Default upper bound on connections handled at once by one server instance.
const DEFAULT_MAX_CONNECTIONS: usize = 1024;

/// Default time to wait for a client to send its request headers.
const DEFAULT_HEADER_TIMEOUT: Duration = Duration::from_secs(5);

/// Server configuration
pub struct ServerConfig {
    /// Address and port the server binds to (always provided to [`ServerConfig::new`])
    pub address: String,
    /// Directory the served files are read from. Defaults to the bundled
    /// `src/www`; change it to serve different content (e.g. per instance).
    pub web_path: String,
    /// Path, relative to [`web_path`], of the default response (the index page)
    /// served when no whitelisted file matches the request
    pub default_file: String,
    /// Content type sent with the default response
    pub default_content_type: String,
    /// Whitelisted URL path -> served file, joined from the MIME table at startup
    pub served_files: HashMap<String, ServedFile>,
    /// Upper bound on connections handled at once. Past this, new connections
    /// wait in the OS accept backlog instead of spawning unbounded tasks. Lives
    /// here (not as a global) so separate server instances can differ.
    pub max_connections: usize,
    /// How long to wait for a client to send its request headers before the
    /// connection is dropped.
    pub header_timeout: Duration,
}

impl ServerConfig {
    /// Build the default configuration for a server bound to `address`. Files are
    /// served from the bundled `src/www`; set [`ServerConfig::web_path`] to serve
    /// a different directory.
    pub fn new(address: String) -> Self {
        // Default return file, relative to the web root
        let default_file = "index.html".to_string();
        let html_content_type = "text/html; charset=UTF-8".to_string();

        // Single source of truth for MIME types of whitelisted files: extension
        // -> (content type, cache). HTML isn't here on purpose: index.html is
        // served as the default response, not via the whitelist, so its type is
        // set directly through `default_content_type` below.
        let mime_types: HashMap<&str, (&str, bool)> = HashMap::from([
            ("ico", ("image/vnd.microsoft.icon", true)),
            ("css", ("text/css; charset=UTF-8", false)),
            ("js", ("text/javascript; charset=UTF-8", false)),
            ("eot", ("application/vnd.ms-fontobject", true)),
            ("svg", ("image/svg+xml", true)),
            ("ttf", ("font/ttf", true)),
            ("woff", ("font/woff", true)),
            ("woff2", ("font/woff2", true)),
        ]);

        // Publicly available files (the whitelist). The content type and cache
        // flag are derived from the MIME table, so they live in exactly one place.
        let public_paths = [
            "/favicon.ico",
            "/styles.css",
            "/script.js",
            "/fonts/web-font.eot",
            "/fonts/web-font.svg",
            "/fonts/web-font.ttf",
            "/fonts/web-font.woff",
            "/fonts/web-font.woff2",
        ];

        // Join the whitelist with the MIME table once at startup. A path whose
        // extension is not registered is reported and skipped, so a forgotten
        // MIME entry fails loudly here instead of silently at request time.
        let mut served_files = HashMap::new();

        for path in public_paths {
            match Self::get_extension_from_filename(path).and_then(|ext| mime_types.get(ext)) {
                Some(&(content_type, cache)) => {
                    served_files.insert(
                        path.to_string(),
                        ServedFile {
                            rel_path: path.trim_start_matches('/').to_string(),
                            content_type: content_type.to_string(),
                            cache,
                        },
                    );
                }
                None => eprintln!("No MIME type registered for '{path}', skipping"),
            }
        }

        Self {
            address,
            web_path: Self::default_web_path(),
            default_file,
            default_content_type: html_content_type,
            served_files,
            max_connections: DEFAULT_MAX_CONNECTIONS,
            header_timeout: DEFAULT_HEADER_TIMEOUT,
        }
    }

    /// Look up the served file matching the request header's first line, if any
    pub fn get_file_data(&self, header: &str) -> Option<&ServedFile> {
        let path = Self::request_path(header)?;
        self.served_files.get(path)
    }

    /// Resolve a path relative to the web root into a full filesystem path.
    pub fn file_path(&self, relative: &str) -> String {
        Self::join_path(&self.web_path, relative)
    }

    /// Location of the bundled web root: the crate's `src/www` directory.
    fn default_web_path() -> String {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("www")
            .to_string_lossy()
            .into_owned()
    }

    /// Join a base directory with a relative path, normalising separators to `/`
    fn join_path(base: &str, relative: &str) -> String {
        Path::new(base)
            .join(relative)
            .to_string_lossy()
            .replace('\\', "/")
    }

    /// Extract the request path (without any query string) from a request line
    /// like `GET /styles.css?v=1 HTTP/1.1`.
    fn request_path(header: &str) -> Option<&str> {
        let path = header.split_whitespace().nth(1)?;
        Some(path.split('?').next().unwrap_or(path))
    }

    /// Get extension by full file name
    fn get_extension_from_filename(filename: &str) -> Option<&str> {
        Path::new(filename).extension().and_then(OsStr::to_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_extension() {
        assert_eq!(
            ServerConfig::get_extension_from_filename("/fonts/web-font.eot"),
            Some("eot")
        );
        assert_eq!(
            ServerConfig::get_extension_from_filename("/no-extension"),
            None
        );
    }

    #[test]
    fn parses_request_path_stripping_query() {
        assert_eq!(
            ServerConfig::request_path("GET /styles.css?v=001 HTTP/1.1\r\n"),
            Some("/styles.css")
        );
        assert_eq!(
            ServerConfig::request_path("GET /index.html HTTP/1.1"),
            Some("/index.html")
        );
        assert_eq!(ServerConfig::request_path("garbage"), None);
    }

    #[test]
    fn serves_whitelisted_file_with_correct_mime() {
        let config = ServerConfig::new("127.0.0.1:0".to_string());
        let entry = config
            .get_file_data("GET /styles.css HTTP/1.1\r\n")
            .expect("styles.css should be served");

        assert_eq!(entry.content_type, "text/css; charset=UTF-8");
        assert!(!entry.cache);
    }

    #[test]
    fn matches_path_with_query_string() {
        let config = ServerConfig::new("127.0.0.1:0".to_string());
        assert!(config
            .get_file_data("GET /fonts/web-font.eot?v=001 HTTP/1.1\r\n")
            .is_some());
    }

    #[test]
    fn ignores_non_whitelisted_path() {
        let config = ServerConfig::new("127.0.0.1:0".to_string());
        assert!(config
            .get_file_data("GET /secret.txt HTTP/1.1\r\n")
            .is_none());
    }
}

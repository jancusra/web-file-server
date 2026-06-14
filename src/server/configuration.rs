//! Server configuration: available files to serving, defined MIME types and some related methods

use std::{collections::HashMap, env, ffi::OsStr, path::Path};

/// A file that is publicly served, with its resolved location and metadata
#[derive(Debug, Clone)]
pub struct ServedFile {
    pub fs_path: String,
    pub content_type: String,
    pub cache: bool,
}

/// Server configuration
pub struct ServerConfig {
    pub default_file: String,
    pub default_content_type: String,
    /// Whitelisted URL path -> served file, joined from the MIME table at startup
    pub served_files: HashMap<String, ServedFile>,
}

impl ServerConfig {
    pub fn init() -> Self {
        let web_path = Self::resolve_web_path();

        // Default return file
        let default_file = Self::join_path(&web_path, "index.html");
        let html_content_type = "text/html; charset=UTF-8".to_string();

        // Single source of truth for MIME types: extension -> (content type, cache)
        let mime_types: HashMap<&str, (&str, bool)> = HashMap::from([
            ("html", (html_content_type.as_str(), false)),
            ("ico", ("image/vnd.microsoft.icon", true)),
            ("css", ("text/css", false)),
            ("js", ("text/javascript", false)),
            ("eot", ("application/vnd.ms-fontobject", true)),
            ("svg", ("image/svg+xml", true)),
            ("ttf", ("font/ttf", true)),
            ("woff", ("font/woff", true)),
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
                            fs_path: Self::join_path(&web_path, path.trim_start_matches('/')),
                            content_type: content_type.to_string(),
                            cache,
                        },
                    );
                }
                None => eprintln!("No MIME type registered for '{path}', skipping"),
            }
        }

        Self {
            default_file,
            default_content_type: html_content_type,
            served_files,
        }
    }

    /// Look up the served file matching the request header's first line, if any
    pub fn get_file_data(&self, header: &str) -> Option<&ServedFile> {
        let path = Self::request_path(header)?;
        self.served_files.get(path)
    }

    /// Web root location: overridable via the `WEB_ROOT` env var, otherwise the
    /// crate's `src/www` directory.
    fn resolve_web_path() -> String {
        if let Ok(path) = env::var("WEB_ROOT") {
            return path;
        }

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
        let config = ServerConfig::init();
        let entry = config
            .get_file_data("GET /styles.css HTTP/1.1\r\n")
            .expect("styles.css should be served");

        assert_eq!(entry.content_type, "text/css");
        assert!(!entry.cache);
    }

    #[test]
    fn matches_path_with_query_string() {
        let config = ServerConfig::init();
        assert!(config
            .get_file_data("GET /fonts/web-font.eot?v=001 HTTP/1.1\r\n")
            .is_some());
    }

    #[test]
    fn ignores_non_whitelisted_path() {
        let config = ServerConfig::init();
        assert!(config
            .get_file_data("GET /secret.txt HTTP/1.1\r\n")
            .is_none());
    }
}

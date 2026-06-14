//! Server configuration: available files to serving, defined MIME types and some related methods

use std::{ffi::OsStr, path::Path};

/// Define MIME type entry
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub extension: String,
    pub content_type: String,
    pub cache: bool,
}

/// Server configuration
pub struct ServerConfig {
    pub web_path: String,
    pub default_file: String,
    pub default_content_type: String,
    pub files_to_serve: Vec<String>,
    pub file_data: Vec<FileEntry>,
}

impl ServerConfig {
    pub fn init() -> Self {
        // Web folder location
        let mut web_path = format!("{}{}", env!("CARGO_MANIFEST_DIR"), "\\src\\www");
        web_path = web_path.replace("\\", "/");

        // Default return file
        let default_file = format!("{}{}", web_path, "/index.html");
        let html_content_type = "text/html; charset=UTF-8".to_string();

        // Publicly available server files
        let files_to_serve = vec![
            "/favicon.ico".to_string(),
            "/styles.css".to_string(),
            "/script.js".to_string(),
            "/fonts/web-font.eot".to_string(),
            "/fonts/web-font.svg".to_string(),
            "/fonts/web-font.ttf".to_string(),
            "/fonts/web-font.woff".to_string(),
        ];

        // The list of all MIME types: (extension, content type, cache)
        let file_data: Vec<FileEntry> = [
            ("html", html_content_type.as_str(), false),
            ("ico", "image/vnd.microsoft.icon", true),
            ("css", "text/css", false),
            ("js", "text/javascript", false),
            ("eot", "application/vnd.ms-fontobject", true),
            ("svg", "image/svg+xml", true),
            ("ttf", "font/ttf", true),
            ("woff", "font/woff", true),
        ]
        .into_iter()
        .map(|(extension, content_type, cache)| FileEntry {
            extension: extension.to_string(),
            content_type: content_type.to_string(),
            cache,
        })
        .collect();

        Self {
            web_path,
            default_file,
            default_content_type: html_content_type,
            files_to_serve,
            file_data,
        }
    }

    // Get the file location and MIME type entry according to the first line of the request header
    pub fn get_file_data(&self, header: &str) -> (Option<String>, Option<FileEntry>) {
        for file in &self.files_to_serve {
            if header.starts_with(&format!("GET {}", file)) {
                if let Some(extension) = Self::get_extension_from_filename(file) {
                    let filter_data: Vec<FileEntry> = self
                        .file_data
                        .iter()
                        .filter(|&fe| fe.extension == extension)
                        .cloned()
                        .collect();
                    return (
                        Some(format!("{}{}", self.web_path, file)),
                        filter_data.first().cloned(),
                    );
                }
            }
        }

        (None, None)
    }

    // Get extension by full file name
    fn get_extension_from_filename(filename: &str) -> Option<&str> {
        Path::new(filename).extension().and_then(OsStr::to_str)
    }
}

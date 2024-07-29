use std::{ffi::OsStr, path::Path};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub extension: String,
    pub content_type: String,
    pub cache: bool
}

pub struct ServerConfig {
    pub web_path: String,
    pub default_file: String,
    pub default_content_type: String,
    pub files_to_serve: Vec<String>,
    pub file_data: Vec<FileEntry>
}

impl ServerConfig {
    pub fn init() -> Self {
        let mut web_path = format!("{}{}", env!("CARGO_MANIFEST_DIR"), "\\src\\www");
        web_path = web_path.replace("\\", "/");

        let default_file = format!("{}{}", web_path, "/index.html".to_string());
        let html_content_type = "text/html; charset=UTF-8".to_string();

        let files_to_serve = vec![
            "/favicon.ico".to_string(),
            "/styles.css".to_string(),
            "/script.js".to_string(),
            "/fonts/web-font.eot".to_string(),
            "/fonts/web-font.svg".to_string(),
            "/fonts/web-font.ttf".to_string(),
            "/fonts/web-font.woff".to_string()
        ];

        let mut file_data: Vec<FileEntry> = vec![];

        file_data.push(FileEntry {
            extension: "html".to_string(),
            content_type: html_content_type.clone(),
            cache: false
        });

        file_data.push(FileEntry {
            extension: "ico".to_string(),
            content_type: "image/vnd.microsoft.icon".to_string(),
            cache: true
        });

        file_data.push(FileEntry {
            extension: "css".to_string(),
            content_type: "text/css".to_string(),
            cache: false
        });

        file_data.push(FileEntry {
            extension: "js".to_string(),
            content_type: "text/javascript".to_string(),
            cache: false
        });

        file_data.push(FileEntry {
            extension: "eot".to_string(),
            content_type: "application/vnd.ms-fontobject".to_string(),
            cache: true
        });

        file_data.push(FileEntry {
            extension: "svg".to_string(),
            content_type: "image/svg+xml".to_string(),
            cache: true
        });

        file_data.push(FileEntry {
            extension: "ttf".to_string(),
            content_type: "font/ttf".to_string(),
            cache: true
        });

        file_data.push(FileEntry {
            extension: "woff".to_string(),
            content_type: "font/woff".to_string(),
            cache: true
        });

        Self {
            web_path,
            default_file,
            default_content_type: html_content_type.clone(),
            files_to_serve,
            file_data
        }
    }

    pub fn get_file_data(&self, header: &str) -> (Option<String>, Option<FileEntry>) {
        for file in &self.files_to_serve {
            if header.starts_with(&format!("GET {}", file)) {
                if let Some(extension) = Self::get_extension_from_filename(file) {
                    let filter_data: Vec<FileEntry> = self.file_data.iter().filter(|&fe| fe.extension == extension).cloned().collect();
                    return (Some(format!("{}{}", self.web_path, file)), filter_data.first().cloned())
                }
            }
        }

        (None, None)
    }

    fn get_extension_from_filename(filename: &str) -> Option<&str> {
        Path::new(filename)
            .extension()
            .and_then(OsStr::to_str)
    }
}
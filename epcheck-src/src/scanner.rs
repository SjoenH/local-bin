use crate::cli::CheckArgs;
use ignore::WalkBuilder;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::task;

/// File scanner for finding source files
pub struct FileScanner {
    cli: CheckArgs,
}

impl FileScanner {
    pub fn new(cli: CheckArgs) -> Self {
        Self { cli }
    }

    /// Find all relevant source files in the directory
    pub fn find_files(&self, dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        // Common source file extensions
        let extensions = [
            "js", "ts", "jsx", "tsx", "py", "rb", "php", "java", "scala", "kt", "swift",
            "go", "rs", "cpp", "c", "h", "hpp", "cs", "fs", "vb", "clj", "cljs", "elm",
            "ex", "exs", "hs", "ml", "fsx", "dart", "lua", "pl", "pm", "tcl", "r",
            "sh", "bash", "zsh", "fish", "ps1", "sql", "xml", "json", "yaml", "yml",
            "toml", "ini", "cfg", "conf", "md", "txt", "html", "htm", "css", "scss",
            "sass", "less", "vue", "svelte", "astro"
        ];

        let mut builder = WalkBuilder::new(dir);
        builder
            .hidden(false) // Include hidden files
            .git_ignore(true) // Respect .gitignore
            .git_global(true) // Respect global gitignore
            .git_exclude(true); // Respect .git/info/exclude

        // Add custom excludes
        for exclude in &self.cli.exclude {
            builder.add_ignore(exclude.clone());
        }

        for result in builder.build() {
            match result {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        if let Some(ext) = entry.path().extension() {
                            if extensions.contains(&ext.to_str().unwrap_or("")) {
                                files.push(entry.path().to_path_buf());
                            }
                        } else if entry.path().file_name()
                            .and_then(|n| n.to_str())
                            .map_or(false, |name| !name.contains('.')) {
                            // Include files without extensions (scripts)
                            files.push(entry.path().to_path_buf());
                        }
                    }
                }
                Err(e) => eprintln!("Warning: {}", e),
            }
        }

        Ok(files)
    }
}

/// Content scanner for finding endpoint usage in files
pub struct ContentScanner {
    endpoint_patterns: Vec<(crate::openapi::Endpoint, Regex)>,
}

impl ContentScanner {
    pub fn new(endpoints: &[crate::openapi::Endpoint]) -> anyhow::Result<Self> {
        let mut patterns = Vec::new();

        for endpoint in endpoints {
            // Create regex patterns for this endpoint
            let method_str = endpoint.method.as_str();

            // Pattern 1: Method calls like client.GET('/api/users')
            let pattern1 = format!(r#"{}\s*\(\s*['"`](/[^'"`]*{})['"`]\s*\)"#,
                                 regex::escape(method_str),
                                 regex::escape(&endpoint.path));

            // Skip pattern 2 for now to avoid false positives

            // Pattern 3: URL patterns with parameters
            let param_pattern = convert_path_to_regex(&endpoint.path);
            let pattern3 = format!(r#"{}\s*\(\s*['"`]({})['"`]\s*\)"#,
                                 regex::escape(method_str),
                                 param_pattern);

            // Pattern 4: Lowercase method calls like api.get('/api/users')
            let lower_method = method_str.to_lowercase();
            let pattern4 = format!(r#"{}\s*\(\s*['"`](/[^'"`]*{})['"`]\s*\)"#,
                                 regex::escape(&lower_method),
                                 regex::escape(&endpoint.path));

            // Pattern 5: Lowercase with parameters
            let pattern5 = format!(r#"{}\s*\(\s*['"`]({})['"`]\s*\)"#,
                                 regex::escape(&lower_method),
                                 param_pattern);

            // Pattern 6: More flexible method calls allowing for additional parameters
            let pattern6 = format!(r#"{}\s*\(\s*['"`]({})['"`]"#,
                                 regex::escape(&lower_method),
                                 regex::escape(&endpoint.path));

            // Pattern 7: Uppercase with additional parameters
            let pattern7 = format!(r#"{}\s*\(\s*['"`]({})['"`]"#,
                                 regex::escape(method_str),
                                 regex::escape(&endpoint.path));



            for pattern in [pattern1, pattern3, pattern4, pattern5, pattern6, pattern7] {
                if let Ok(regex) = Regex::new(&pattern) {
                    patterns.push((endpoint.clone(), regex));
                }
            }
        }

        Ok(Self {
            endpoint_patterns: patterns,
        })
    }

    /// Scan a file for endpoint usage
    pub fn scan_file(&self, path: &Path) -> anyhow::Result<Vec<(crate::openapi::Endpoint, usize)>> {
        let content = std::fs::read_to_string(path)?;
        let mut found_endpoints = std::collections::HashMap::new();

        // Debug: print file content for files
        if path.to_string_lossy().contains("traditional") || path.to_string_lossy().contains("userService") {
            eprintln!("DEBUG: Scanning file: {}", path.display());
            eprintln!("DEBUG: Content contains:");
            for line in content.lines() {
                if line.contains("api.") {
                    eprintln!("  {}", line.trim());
                }
            }
        }

        for (endpoint, regex) in &self.endpoint_patterns {
            let count = regex.find_iter(&content).count();
            if count > 0 {
                eprintln!("DEBUG: Found {} matches for {} {} in {}", count, endpoint.method.as_str(), endpoint.path, path.display());
                *found_endpoints.entry(endpoint.clone()).or_insert(0) += count;
            }
        }

        Ok(found_endpoints.into_iter().collect())
    }

    /// Scan multiple files concurrently and return detailed usage information
    pub async fn scan_files(&self, files: Vec<PathBuf>) -> anyhow::Result<HashMap<crate::openapi::Endpoint, (usize, Vec<String>)>> {
        let mut handles = Vec::new();
        let patterns = &self.endpoint_patterns;

        for file in files {
            let patterns_clone = patterns.clone();
            let handle = task::spawn(async move {
                let mut file_results = Vec::new();
                if let Ok(content) = tokio::fs::read_to_string(&file).await {
                        for (endpoint, regex) in &patterns_clone {
                        let count = regex.find_iter(&content).count();
                        if count > 0 {
                            file_results.push((endpoint.clone(), count));
                        }
                    }
                }
                (file, file_results)
            });
            handles.push(handle);
        }

        let mut all_results: HashMap<crate::openapi::Endpoint, (usize, std::collections::HashSet<String>)> = HashMap::new();

        for handle in handles {
            let (file_path, file_results) = handle.await?;
            let file_name = file_path.to_string_lossy().to_string();

            for (endpoint, count) in file_results {
                let (total_count, files) = all_results.entry(endpoint).or_insert((0, std::collections::HashSet::new()));
                *total_count += count;
                files.insert(file_name.clone());
            }
        }

        // Convert HashSet to Vec for the final result
        let mut final_results: HashMap<crate::openapi::Endpoint, (usize, Vec<String>)> = HashMap::new();
        for (endpoint, (total_count, files_set)) in all_results {
            let mut files_vec: Vec<String> = files_set.into_iter().collect();
            files_vec.sort(); // Sort for consistent output
            final_results.insert(endpoint, (total_count, files_vec));
        }

        Ok(final_results)
    }
}

/// Convert OpenAPI path with parameters to regex pattern
fn convert_path_to_regex(path: &str) -> String {
    // Escape special regex characters except {}
    let escaped = regex::escape(path);

    // Convert {param} to [^/]+ (one or more non-slash characters)
    let param_regex = Regex::new(r"\\\{[^}]+\}").unwrap();
    param_regex.replace_all(&escaped, r"[^/]+").to_string()
}
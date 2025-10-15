use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI specification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    #[serde(rename = "openapi")]
    pub version: Option<String>,
    pub info: Option<Info>,
    pub paths: HashMap<String, PathItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub title: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    #[serde(flatten)]
    pub operations: HashMap<String, Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operation_id: Option<String>,
    #[serde(flatten)]
    pub parameters: HashMap<String, serde_json::Value>,
}

/// HTTP methods supported by OpenAPI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Trace,
}

impl HttpMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "get" => Some(Self::Get),
            "post" => Some(Self::Post),
            "put" => Some(Self::Put),
            "delete" => Some(Self::Delete),
            "patch" => Some(Self::Patch),
            "head" => Some(Self::Head),
            "options" => Some(Self::Options),
            "trace" => Some(Self::Trace),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Trace => "TRACE",
        }
    }
}

/// Endpoint definition combining path and method
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Endpoint {
    pub path: String,
    pub method: HttpMethod,
}

impl Endpoint {
    pub fn new(path: String, method: HttpMethod) -> Self {
        Self { path, method }
    }
}

impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.method.as_str(), self.path)
    }
}

/// Extract all endpoints from an OpenAPI specification
pub fn extract_endpoints(spec: &OpenApiSpec) -> Vec<Endpoint> {
    let mut endpoints = Vec::new();

    for (path, path_item) in &spec.paths {
        for (method_str, _operation) in &path_item.operations {
            if let Some(method) = HttpMethod::from_str(method_str) {
                endpoints.push(Endpoint::new(path.clone(), method));
            }
        }
    }

    endpoints
}
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::auth::AuthConfig;

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Custom(String),
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Patch => write!(f, "PATCH"),
            Method::Delete => write!(f, "DELETE"),
            Method::Head => write!(f, "HEAD"),
            Method::Options => write!(f, "OPTIONS"),
            Method::Custom(raw) => write!(f, "{raw}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyValue {
    pub name: String,
    pub value: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BodyKind {
    #[default]
    None,
    Raw {
        content_type: String,
        content: String,
    },
    FormUrlEncoded {
        fields: Vec<KeyValue>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMeta {
    pub id: Uuid,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestFile {
    pub meta: RequestMeta,
    #[serde(default)]
    pub method: Method,
    pub url: String,
    #[serde(default)]
    pub query_params: Vec<KeyValue>,
    #[serde(default)]
    pub headers: Vec<KeyValue>,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub body: BodyKind,
}

impl RequestFile {
    /// An unsaved, in-memory request used before the user has created or
    /// loaded a file from a collection (keeps the URL-bar-only flow working
    /// standalone).
    pub fn scratch() -> Self {
        Self::scratch_named("untitled")
    }

    pub fn scratch_named(name: &str) -> Self {
        Self {
            meta: RequestMeta {
                id: Uuid::new_v4(),
                name: name.to_string(),
                description: None,
            },
            method: Method::default(),
            url: String::new(),
            query_params: Vec::new(),
            headers: Vec::new(),
            auth: AuthConfig::default(),
            body: BodyKind::default(),
        }
    }
}

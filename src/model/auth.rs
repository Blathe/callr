use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthConfig {
    #[default]
    None,
    Bearer {
        token: String,
    },
    Basic {
        username: String,
        password: String,
    },
    ApiKey {
        name: String,
        value: String,
        location: ApiKeyLocation,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyLocation {
    Header,
    Query,
}

impl ApiKeyLocation {
    pub fn label(&self) -> &'static str {
        match self {
            ApiKeyLocation::Header => "header",
            ApiKeyLocation::Query => "query",
        }
    }
}

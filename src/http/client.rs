use reqwest::{Client, Method as ReqMethod, RequestBuilder};

use crate::model::auth::{ApiKeyLocation, AuthConfig};
use crate::model::request::{BodyKind, Method, RequestFile};

pub fn build_request(client: &Client, request: &RequestFile) -> RequestBuilder {
    let method = to_reqwest_method(&request.method);
    let mut builder = client.request(method, &request.url);

    for header in &request.headers {
        if header.enabled && !header.name.is_empty() {
            builder = builder.header(&header.name, &header.value);
        }
    }

    for param in &request.query_params {
        if param.enabled && !param.name.is_empty() {
            builder = builder.query(&[(&param.name, &param.value)]);
        }
    }

    if let BodyKind::Raw {
        content_type,
        content,
    } = &request.body
    {
        builder = builder
            .header(reqwest::header::CONTENT_TYPE, content_type.clone())
            .body(content.clone());
    }

    apply_auth(builder, &request.auth)
}

fn apply_auth(builder: RequestBuilder, auth: &AuthConfig) -> RequestBuilder {
    match auth {
        AuthConfig::None => builder,
        AuthConfig::Bearer { token } => builder.bearer_auth(token),
        AuthConfig::Basic { username, password } => builder.basic_auth(username, Some(password)),
        AuthConfig::ApiKey {
            name,
            value,
            location,
        } => match location {
            ApiKeyLocation::Header => builder.header(name, value),
            ApiKeyLocation::Query => builder.query(&[(name, value)]),
        },
    }
}

fn to_reqwest_method(method: &Method) -> ReqMethod {
    match method {
        Method::Get => ReqMethod::GET,
        Method::Post => ReqMethod::POST,
        Method::Put => ReqMethod::PUT,
        Method::Patch => ReqMethod::PATCH,
        Method::Delete => ReqMethod::DELETE,
        Method::Head => ReqMethod::HEAD,
        Method::Options => ReqMethod::OPTIONS,
        Method::Custom(raw) => ReqMethod::from_bytes(raw.as_bytes()).unwrap_or(ReqMethod::GET),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::request::RequestFile;

    fn scratch_request(url: &str) -> RequestFile {
        let mut request = RequestFile::scratch();
        request.url = url.to_string();
        request
    }

    #[test]
    fn bearer_auth_sets_authorization_header() {
        let client = Client::new();
        let mut request = scratch_request("https://example.com/");
        request.auth = AuthConfig::Bearer {
            token: "abc123".to_string(),
        };
        let built = build_request(&client, &request).build().unwrap();
        assert_eq!(built.headers().get("authorization").unwrap(), "Bearer abc123");
    }

    #[test]
    fn basic_auth_sets_authorization_header() {
        let client = Client::new();
        let mut request = scratch_request("https://example.com/");
        request.auth = AuthConfig::Basic {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        let built = build_request(&client, &request).build().unwrap();
        let header = built.headers().get("authorization").unwrap().to_str().unwrap();
        assert!(header.starts_with("Basic "));
    }

    #[test]
    fn api_key_header_location_sets_custom_header() {
        let client = Client::new();
        let mut request = scratch_request("https://example.com/");
        request.auth = AuthConfig::ApiKey {
            name: "X-Api-Key".to_string(),
            value: "secret".to_string(),
            location: ApiKeyLocation::Header,
        };
        let built = build_request(&client, &request).build().unwrap();
        assert_eq!(built.headers().get("X-Api-Key").unwrap(), "secret");
    }

    #[test]
    fn api_key_query_location_appends_query_param() {
        let client = Client::new();
        let mut request = scratch_request("https://example.com/");
        request.auth = AuthConfig::ApiKey {
            name: "api_key".to_string(),
            value: "secret".to_string(),
            location: ApiKeyLocation::Query,
        };
        let built = build_request(&client, &request).build().unwrap();
        assert!(built.url().query().unwrap().contains("api_key=secret"));
    }

    // Live checks against httpbin.org — not run by default (`cargo test` stays
    // network-independent); run with `cargo test -- --ignored` to verify the
    // three auth types actually work end-to-end against a real server.
    #[tokio::test]
    #[ignore]
    async fn live_bearer_auth_succeeds() {
        let client = Client::new();
        let mut request = scratch_request("https://httpbin.org/bearer");
        request.auth = AuthConfig::Bearer {
            token: "testtoken".to_string(),
        };
        let response = build_request(&client, &request).send().await.unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    #[ignore]
    async fn live_basic_auth_succeeds() {
        let client = Client::new();
        let mut request = scratch_request("https://httpbin.org/basic-auth/user/pass");
        request.auth = AuthConfig::Basic {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        let response = build_request(&client, &request).send().await.unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    #[ignore]
    async fn live_api_key_header_succeeds() {
        let client = Client::new();
        let mut request = scratch_request("https://httpbin.org/headers");
        request.auth = AuthConfig::ApiKey {
            name: "X-Api-Key".to_string(),
            value: "secret".to_string(),
            location: ApiKeyLocation::Header,
        };
        let response = build_request(&client, &request).send().await.unwrap();
        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["headers"]["X-Api-Key"], "secret");
    }

    #[tokio::test]
    #[ignore]
    async fn live_api_key_query_succeeds() {
        let client = Client::new();
        let mut request = scratch_request("https://httpbin.org/get");
        request.auth = AuthConfig::ApiKey {
            name: "api_key".to_string(),
            value: "secret".to_string(),
            location: ApiKeyLocation::Query,
        };
        let response = build_request(&client, &request).send().await.unwrap();
        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["args"]["api_key"], "secret");
    }
}

use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use secrecy::ExposeSecret;
use serde::Serialize;

use crate::{config::Config, error::CliError};

/// Async HTTP client pre-configured for a single Home Assistant instance.
///
/// Wraps [`reqwest::Client`] with:
/// - The `Authorization: Bearer <token>` header set on every request.
/// - A base URL prefix prepended to every path.
///
/// Construct with [`HaClient::new`], then call the typed request methods.
pub struct HaClient {
    client: Client,
    /// Normalised base URL (no trailing slash), e.g. `http://homeassistant.local:8123`.
    base_url: String,
}

impl HaClient {
    /// Creates a new [`HaClient`] from a resolved [`Config`].
    ///
    /// Builds a [`reqwest::Client`] with the `Authorization` bearer token
    /// pre-set as a default header so it is sent on every request without
    /// repetition at each call site.
    ///
    /// # Errors
    ///
    /// Returns [`CliError::InvalidHeader`] if the token contains bytes that are
    /// illegal in an HTTP header value, or [`CliError::Http`] if the underlying
    /// `reqwest` client cannot be constructed.
    pub fn new(config: &Config) -> Result<Self, CliError> {
        let auth_value =
            HeaderValue::from_str(&format!("Bearer {}", config.token.expose_secret()))?;

        let mut default_headers = HeaderMap::new();
        default_headers.insert(AUTHORIZATION, auth_value);

        let client = Client::builder().default_headers(default_headers).build()?;

        Ok(Self {
            client,
            base_url: config.url.clone(),
        })
    }

    /// Constructs the full URL for an API path.
    ///
    /// # Arguments
    ///
    /// * `path` — path segment starting with `/`, e.g. `/states`.
    fn url(&self, path: &str) -> String {
        format!("{}/api{}", self.base_url, path)
    }

    // ------------------------------------------------------------------
    // Internal response handlers
    // ------------------------------------------------------------------

    /// Interprets a response as JSON, surfacing non-2xx status as an error.
    async fn handle_json_response(
        &self,
        response: reqwest::Response,
    ) -> Result<serde_json::Value, CliError> {
        let status = response.status();
        if status.is_success() {
            let text = response.text().await?;
            // Some endpoints (e.g. DELETE) return an empty body on success.
            if text.is_empty() {
                return Ok(serde_json::Value::Null);
            }
            Ok(serde_json::from_str(&text)?)
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("(no response body)"));
            Err(CliError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    /// Interprets a response as plain text, surfacing non-2xx status as an error.
    async fn handle_text_response(&self, response: reqwest::Response) -> Result<String, CliError> {
        let status = response.status();
        if status.is_success() {
            Ok(response.text().await?)
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("(no response body)"));
            Err(CliError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    // ------------------------------------------------------------------
    // Public request methods
    // ------------------------------------------------------------------

    /// Sends `GET /api{path}` and returns the response body parsed as JSON.
    ///
    /// # Errors
    ///
    /// Propagates network errors ([`CliError::Http`]), JSON parse errors
    /// ([`CliError::Json`]), or API errors ([`CliError::Api`]).
    pub async fn get_json(&self, path: &str) -> Result<serde_json::Value, CliError> {
        tracing::debug!(path, "GET (json)");
        let response = self.client.get(self.url(path)).send().await?;
        self.handle_json_response(response).await
    }

    /// Sends `GET /api{path}` and returns the response body as a plain string.
    ///
    /// Use this for endpoints that return non-JSON text (e.g. `/api/error_log`).
    ///
    /// # Errors
    ///
    /// Propagates network or API errors.
    pub async fn get_text(&self, path: &str) -> Result<String, CliError> {
        tracing::debug!(path, "GET (text)");
        let response = self.client.get(self.url(path)).send().await?;
        self.handle_text_response(response).await
    }

    /// Sends `GET /api{path}` with URL query parameters and returns JSON.
    ///
    /// # Arguments
    ///
    /// * `query` — any type accepted by [`reqwest::RequestBuilder::query`],
    ///   typically `&[(&str, &str)]`.
    ///
    /// # Errors
    ///
    /// Propagates network, JSON parse, or API errors.
    pub async fn get_json_with_params<Q>(
        &self,
        path: &str,
        params: &Q,
    ) -> Result<serde_json::Value, CliError>
    where
        Q: Serialize + ?Sized,
    {
        tracing::debug!(path, "GET (json, with params)");
        let response = self.client.get(self.url(path)).query(params).send().await?;
        self.handle_json_response(response).await
    }

    /// Sends `POST /api{path}` with a JSON body and returns JSON.
    ///
    /// # Errors
    ///
    /// Propagates network, JSON parse, or API errors.
    pub async fn post_json(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, CliError> {
        tracing::debug!(path, "POST (json body)");
        let response = self.client.post(self.url(path)).json(body).send().await?;
        self.handle_json_response(response).await
    }

    /// Sends `POST /api{path}` with a JSON body **and** URL query parameters,
    /// returning JSON.
    ///
    /// Used by `service call --return-response` which appends `?return_response`.
    ///
    /// # Errors
    ///
    /// Propagates network, JSON parse, or API errors.
    pub async fn post_json_with_params<Q>(
        &self,
        path: &str,
        body: &serde_json::Value,
        params: &Q,
    ) -> Result<serde_json::Value, CliError>
    where
        Q: Serialize + ?Sized,
    {
        tracing::debug!(path, "POST (json body, with params)");
        let response = self
            .client
            .post(self.url(path))
            .query(params)
            .json(body)
            .send()
            .await?;
        self.handle_json_response(response).await
    }

    /// Sends `POST /api{path}` with **no body** and returns JSON.
    ///
    /// Used by endpoints that take no input (e.g. `check_config`).
    ///
    /// # Errors
    ///
    /// Propagates network, JSON parse, or API errors.
    pub async fn post_empty(&self, path: &str) -> Result<serde_json::Value, CliError> {
        tracing::debug!(path, "POST (no body)");
        let response = self.client.post(self.url(path)).send().await?;
        self.handle_json_response(response).await
    }

    /// Sends `POST /api{path}` with a JSON body and returns the response as plain text.
    ///
    /// Used by `/api/template` which renders a Jinja2 string and returns the result
    /// as plain text rather than JSON.
    ///
    /// # Errors
    ///
    /// Propagates network or API errors.
    pub async fn post_text(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<String, CliError> {
        tracing::debug!(path, "POST (json body, text response)");
        let response = self.client.post(self.url(path)).json(body).send().await?;
        self.handle_text_response(response).await
    }

    /// Sends `DELETE /api{path}` and returns JSON (or `Null` on empty body).
    ///
    /// # Errors
    ///
    /// Propagates network, JSON parse, or API errors.
    pub async fn delete(&self, path: &str) -> Result<serde_json::Value, CliError> {
        tracing::debug!(path, "DELETE");
        let response = self.client.delete(self.url(path)).send().await?;
        self.handle_json_response(response).await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::config::Config;
    use secrecy::SecretString;

    /// Helper: build a client pointing at the mockito server.
    fn make_client(server_url: &str) -> HaClient {
        let config = Config {
            url: server_url.trim_end_matches('/').to_string(),
            token: SecretString::new("test-token".into()),
        };
        HaClient::new(&config).expect("client construction should succeed")
    }

    #[tokio::test]
    async fn get_json_parses_response() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"message":"API running."}"#)
            .create_async()
            .await;

        let client = make_client(&server.url());
        let value = client.get_json("/").await.expect("GET should succeed");
        assert_eq!(value["message"], "API running.");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn non_2xx_returns_api_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/states/bad.entity")
            .with_status(404)
            .with_body("Entity not found")
            .create_async()
            .await;

        let client = make_client(&server.url());
        let result = client.get_json("/states/bad.entity").await;
        assert!(matches!(result, Err(CliError::Api { status: 404, .. })));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn delete_handles_empty_body() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/api/states/sensor.test")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let client = make_client(&server.url());
        let value = client
            .delete("/states/sensor.test")
            .await
            .expect("DELETE should succeed");
        assert_eq!(value, serde_json::Value::Null);
        mock.assert_async().await;
    }
}

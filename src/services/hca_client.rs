use anyhow::Result;
use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl, RedirectUrl, basic::BasicClient, AuthorizationCode, CsrfToken};
use oauth2::reqwest::async_http_client;
use serde::Deserialize;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

#[derive(Deserialize, Debug)]
pub struct HcaIdentity {
    pub id: String,
    pub email: Option<String>,
    pub slack_id: Option<String>,
    pub display_name: Option<String>,
}

pub struct HcaClient {
    pub host: String,
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
}

impl HcaClient {
    pub fn new(host: String, client_id: String, client_secret: String, callback_url: String) -> Self {
        Self { host, client_id, client_secret, callback_url }
    }

    pub fn auth_url(&self, state: &str) -> String {
        
        
        
        if std::env::var("MOCK_HCA").ok().as_deref() == Some("1") {
            
            return format!("{}?code=MOCKCODE&state={}", self.callback_url, state);
        }

        let auth_url = AuthUrl::new(format!("{}/oauth/authorize", self.host)).unwrap();
        let token_url = TokenUrl::new(format!("{}/oauth/token", self.host)).unwrap();
        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            auth_url,
            Some(token_url),
        ).set_redirect_uri(RedirectUrl::new(self.callback_url.clone()).unwrap());

        let (authorize_url, _csrf_token) = client
            .authorize_url(|| CsrfToken::new(state.to_string()))
            .add_scope(oauth2::Scope::new("identity".to_string()))
            .url();
        authorize_url.to_string()
    }

    pub async fn exchange_code(&self, code: &str) -> Result<(String, Option<String>)> {
        
        if std::env::var("MOCK_HCA").ok().as_deref() == Some("1") {
            return Ok(("mock_access_token".to_string(), Some("mock_refresh".to_string())));
        }

        let auth_url = AuthUrl::new(format!("{}/oauth/authorize", self.host)).unwrap();
        let token_url = TokenUrl::new(format!("{}/oauth/token", self.host)).unwrap();
        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            auth_url,
            Some(token_url),
        ).set_redirect_uri(RedirectUrl::new(self.callback_url.clone()).unwrap());

        use oauth2::TokenResponse;
        let token_result = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(async_http_client)
            .await?;

        let access_token = token_result.access_token().secret().to_string();
        let refresh_token = token_result.refresh_token().map(|r| r.secret().to_string());
        Ok((access_token, refresh_token))
    }

    pub async fn fetch_me(&self, access_token: &str) -> Result<HcaIdentity> {
        
        if std::env::var("MOCK_HCA").ok().as_deref() == Some("1") {
            return Ok(HcaIdentity {
                id: "mock-id".to_string(),
                email: Some("dev@example.com".to_string()),
                slack_id: Some("DEVSLACK".to_string()),
                display_name: Some("Dev User".to_string()),
            });
        }

        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/me", self.host);
        let resp = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            anyhow::bail!("HCA /me returned {}: {}", status, text);
        }
        let me: HcaIdentity = serde_json::from_str(&text)?;
        Ok(me)
    }
}

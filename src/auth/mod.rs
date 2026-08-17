use crate::auth::errors::AuthErrors;
use crate::auth::errors::AuthErrors::{OAuth2, XboxLive};
use crate::auth::structs::{MinecraftAuthResponse, OAuthTokenResponse};
use reqwest::{Body, Url};
use serde_json::json;
use tiny_http::{Response, Server};

mod errors;
mod structs;

pub struct Authenticator {
    oauth_token_response: OAuthTokenResponse,
    xbl_token: String,
    user_hash: String,
}

impl Authenticator {
    pub async fn exchange_code_for_token(
        client_id: &str,
        auth_code: &str,
    ) -> Result<OAuthTokenResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let params = [
            ("client_id", client_id),
            ("scope", "XboxLive.signin offline_access"),
            ("code", auth_code),
            ("redirect_uri", "http://localhost:8080/redirect"),
            ("grant_type", "authorization_code"),
        ];

        let token_res = client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&params)
            .send()
            .await?
            .json::<OAuthTokenResponse>()
            .await?;

        Ok(token_res)
    }
    pub fn request_auth_code(client_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let redirect_uri = "http://localhost:8080/redirect";

        let mut auth_url =
            Url::parse("https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize")?;
        auth_url
            .query_pairs_mut()
            .append_pair("client_id", client_id)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("response_mode", "query")
            .append_pair("scope", "XboxLive.signin offline_access")
            .append_pair("state", "RANDOM_NONCE");

        let server = Server::http("127.0.0.1:8080").map_err(|e| e.to_string())?;

        open::that(auth_url.as_str())?;

        println!("Waiting for user login in browser...");

        for request in server.incoming_requests() {
            let url_path = request.url();
            let parsed_url = Url::parse(&format!("http://localhost:8080{}", url_path))?;

            if parsed_url.path() == "/redirect" {
                let code = parsed_url
                    .query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, val)| val.into_owned());

                if let Some(auth_code) = code {
                    let response =
                        Response::from_string("Authentication successful! You can close this tab.");
                    request.respond(response)?;
                    return Ok(auth_code);
                }
            }
        }

        Err("Failed to capture authorization code".into())
    }

    pub async fn auth_xbox_live(token: &str) -> Result<MinecraftAuthResponse, AuthErrors> {
        let client = reqwest::Client::new();

        let body = json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={token}")
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        })
        .to_string();

        client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(Body::wrap(body))
            .send()
            .await
            .map_err(|e| XboxLive(e.to_string()))?
            .json::<MinecraftAuthResponse>()
            .await
            .map_err(|e| XboxLive(e.to_string()))
    }

    pub async fn auth_xsts(token: &str) -> Result<MinecraftAuthResponse, AuthErrors> {
        let client = reqwest::Client::new();

        let body = json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [
                    format!("{token}")
                ]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        })
            .to_string();

        client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(Body::wrap(body))
            .send()
            .await
            .map_err(|e| XboxLive(e.to_string()))?
            .json::<MinecraftAuthResponse>()
            .await
            .map_err(|e| XboxLive(e.to_string()))
    }
}

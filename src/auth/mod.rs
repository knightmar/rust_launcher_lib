use crate::auth::errors::AuthErrors;
use crate::auth::structs::{
    MinecraftAuthResponse, MinecraftProfile, MinecraftStoreResponse, OAuthTokenResponse,
    XboxLiveResponse, XstsError,
};
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use reqwest::{Body, Url};
use rsa::pkcs8::DecodePublicKey;
use rsa::signature::digest::Digest;
use rsa::{Pkcs1v15Sign, RsaPublicKey};
use serde_json::json;
use sha2::Sha256;
use std::time::Duration;
use tiny_http::{Response, Server};

mod errors;
mod structs;

const MOJANG_PUBLIC_KEY_PEM: &str = r#"-----BEGIN PUBLIC KEY-----
MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAtz7jy4jRH3psj5AbVS6W
NHjniqlr/f5JDly2M8OKGK81nPEq765tJuSILOWrC3KQRvHJIhf84+ekMGH7iGlO
4DPGDVb6hBGoMMBhCq2jkBjuJ7fVi3oOxy5EsA/IQqa69e55ugM+GJKUndLyHeNn
X6RzRzDT4tX/i68WJikwL8rR8Jq49aVJlIEFT6F+1rDQdU2qcpfT04CBYLM5gMxE
fWRl6u1PNQixz8vSOv8pA6hB2DU8Y08VvbK7X2ls+BiS3wqqj3nyVWqoxrwVKiXR
kIqIyIAedYDFSaIq5vbmnVtIonWQPeug4/0spLQoWnTUpXRZe2/+uAKN1RY9mmaB
pRFV/Osz3PDOoICGb5AZ0asLFf/qEvGJ+di6Ltt8/aaoBuVw+7fnTw2BhkhSq1S/
va6LxHZGXE9wsLj4CN8mZXHfwVD9QG0VNQTUgEGZ4ngf7+0u30p7mPt5sYy3H+Fm
sWXqFZn55pecmrgNLqtETPWMNpWc2fJu/qqnxE9o2tBGy/MqJiw3iLYxf7U+4le4
jM49AUKrO16bD1rdFwyVuNaTefObKjEMTX9gyVUF6o7oDEItp5NHxFm3CqnQRmch
HsMs+NxEnN4E9a8PDB23b4yjKOQ9VHDxBxuaZJU60GBCIOF9tslb7OAkheSJx5Xy
EYblHbogFGPRFU++NrSQRX0CAwEAAQ==
-----END PUBLIC KEY-----"#;

pub struct Authenticator {}

impl Authenticator {
    fn client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap()
    }

    pub async fn exchange_code_for_token(
        client_id: &str,
        auth_code: &str,
    ) -> Result<OAuthTokenResponse, AuthErrors> {
        let params = [
            ("client_id", client_id),
            ("scope", "XboxLive.signin offline_access"),
            ("code", auth_code),
            ("redirect_uri", "http://localhost:8080/redirect"),
            ("grant_type", "authorization_code"),
        ];

        let token_res = Self::client()
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&params)
            .send()
            .await.map_error(|e| {AuthErrors::OAuth2(e.to_string)})?
            .json::<OAuthTokenResponse>()
            .await.map_error(|e| {AuthErrors::OAuth2(e.to_string)})?;

        Ok(token_res)
    }
    pub fn auth_oauth2(client_id: &str) -> Result<String, AuthErrors> {
        const PORT: u16 = 8080;
        const MAX_RETRIES: u8 = 5;

        let redirect_uri = format!("http://localhost:{PORT}/redirect");
        let mut attempts = 0;

        let state = rand::random::<u64>().to_string();

        let mut auth_url =
            Url::parse("https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize")
                .map_err(|e| AuthErrors::OAuth2(e.to_string()))?;
        auth_url
            .query_pairs_mut()
            .append_pair("client_id", client_id)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("response_mode", "query")
            .append_pair("scope", "XboxLive.signin offline_access")
            .append_pair("state", &state);

        let server = loop {
            match Server::http(&format!("127.0.0.1:{}", PORT)) {
                Ok(s) => break s,
                Err(e) => {
                    attempts += 1;
                    if attempts >= MAX_RETRIES {
                        return Err(AuthErrors::OAuth2(
                            format!(
                                "Port {} is busy after {} attempts: {}",
                                PORT, MAX_RETRIES, e
                            )
                            .into(),
                        ));
                    }
                    println!(
                        "Port {} is busy, retrying in 1s... (attempt {}/{})",
                        PORT,
                        attempts + 1,
                        MAX_RETRIES
                    );
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        };
        open::that(auth_url.as_str()).map_err(|e| AuthErrors::OAuth2(e.to_string()))?;

        println!("Waiting for user login in browser...");

        for request in server.incoming_requests() {
            let url_path = request.url();
            let parsed_url = Url::parse(&format!("http://localhost:8080{}", url_path))
                .map_err(|e| AuthErrors::OAuth2(e.to_string()))?;

            if let Some(state_param) = parsed_url.query_pairs().find(|(k, _)| k == "state") {
                if state_param.1 != state {
                    return Err(AuthErrors::OAuth2("Invalid state".to_string()));
                }
            }

            if parsed_url.path() == "/redirect" {
                let code = parsed_url
                    .query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, val)| val.into_owned());

                if let Some(auth_code) = code {
                    let response =
                        Response::from_string("Authentication successful! You can close this tab.");
                    request
                        .respond(response)
                        .map_err(|e| AuthErrors::OAuth2(e.to_string()))?;
                    return Ok(auth_code);
                }
            }
        }

        Err(AuthErrors::OAuth2(
            "Failed to capture authorization code".into(),
        ))
    }

    pub async fn auth_xbox_live(oauth2_access_token: &str) -> Result<XboxLiveResponse, AuthErrors> {
        let body = json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={oauth2_access_token}")
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        })
        .to_string();

        Self::client()
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(Body::wrap(body))
            .send()
            .await
            .map_err(|e| AuthErrors::XboxLive(e.to_string()))?
            .json::<XboxLiveResponse>()
            .await
            .map_err(|e| AuthErrors::XboxLive(e.to_string()))
    }

    pub async fn auth_xsts(xbl_token: &str) -> Result<XboxLiveResponse, AuthErrors> {
        let body = json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [
                    xbl_token
                ]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        })
        .to_string();

        Self::client()
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(Body::wrap(body))
            .send()
            .await
            .map_err(|e| {
                AuthErrors::Xsts(
                    serde_json::from_str::<XstsError>(&e.to_string())
                        .map(|t| t.message)
                        .unwrap_or_else(|e| e.to_string()),
                )
            })?
            .json::<XboxLiveResponse>()
            .await
            .map_err(|e| AuthErrors::Xsts(e.to_string()))
    }

    pub async fn auth_minecraft(
        userhash: &str,
        xsts_token: &str,
    ) -> Result<MinecraftAuthResponse, AuthErrors> {
        let body = json!({
            "identityToken": format!("XBL3.0 x={userhash};{xsts_token}")
        })
        .to_string();

        Self::client()
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(Body::wrap(body))
            .send()
            .await
            .map_err(|e| AuthErrors::Minecraft(e.to_string()))?
            .json::<MinecraftAuthResponse>()
            .await
            .map_err(|e| AuthErrors::Minecraft(e.to_string()))
    }

    pub fn verify_jwt_signature(jwt: &str) -> Result<bool, AuthErrors> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Ok(false);
        }

        let header_b64 = parts[0];
        let payload_b64 = parts[1];
        let signature_b64 = parts[2];

        let signed_data = format!("{}.{}", header_b64, payload_b64);

        let public_key = RsaPublicKey::from_public_key_pem(MOJANG_PUBLIC_KEY_PEM)
            .map_err(|e| AuthErrors::OAuth2(e.to_string()))?;

        let signature_bytes = BASE64_URL_SAFE_NO_PAD
            .decode(signature_b64)
            .map_err(|e| AuthErrors::OAuth2(e.to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(signed_data.as_bytes());
        let hashed_data = hasher.finalize();

        let scheme = Pkcs1v15Sign::new::<Sha256>();
        let is_valid = public_key
            .verify(scheme, &hashed_data, &signature_bytes)
            .is_ok();

        Ok(is_valid)
    }

    pub async fn check_game_ownership(minecraft_access_token: &str) -> Result<bool, AuthErrors> {
        let response = Self::client()
            .get("https://api.minecraftservices.com/entitlements/mcstore")
            .header("Authorization", format!("Bearer {minecraft_access_token}"))
            .send()
            .await
            .map_err(|e| AuthErrors::Minecraft(e.to_string()))?
            .json::<MinecraftStoreResponse>()
            .await
            .map_err(|e| AuthErrors::Minecraft(e.to_string()))?;

        let is_valid = Self::verify_jwt_signature(&response.signature)?;

        if !is_valid {
            return Err(AuthErrors::Minecraft(
                "Invalid signature when checking ownership".to_string(),
            ));
        }

        Ok(!response.items.is_empty())
    }

    pub async fn get_minecraft_profile(
        minecraft_access_token: &str,
    ) -> Result<MinecraftProfile, AuthErrors> {
        let response = Self::client()
            .get("https://api.minecraftservices.com/minecraft/profile")
            .header("Authorization", format!("Bearer {minecraft_access_token}"))
            .send()
            .await
            .map_err(|e| AuthErrors::Minecraft(e.to_string()))?;

        let body_text = response
            .text()
            .await
            .map_err(|e| AuthErrors::Minecraft(e.to_string()))?;

        let profile = serde_json::from_str::<MinecraftProfile>(&body_text)
            .map_err(|e| AuthErrors::Minecraft(format!("Can't parse JSON: {e}")))?;

        Ok(profile)
    }

    pub async fn auth(client_id: &str) -> Result<MinecraftProfile, AuthErrors> {
        let oauth2_code = Self::auth_oauth2(client_id)?;
        let oauth_token_response = Self::exchange_code_for_token(client_id, &oauth2_code)
            .await
            .map_err(|e| AuthErrors::OAuth2(e.to_string()))?;
        let xbox_live_response =
            Self::auth_xbox_live(oauth_token_response.access_token.as_str()).await?;
        let xsts_response = Self::auth_xsts(xbox_live_response.token.as_str()).await?;
        let minecraft_response = Self::auth_minecraft(
            xsts_response.display_claims.xui[0].uhs.as_str(),
            xsts_response.token.as_str(),
        )
        .await?;
        if !Self::check_game_ownership(minecraft_response.access_token.as_str()).await? {
            return Err(AuthErrors::Minecraft("Game not owned".to_string()));
        }
        Self::get_minecraft_profile(minecraft_response.access_token.as_str()).await
    }
}

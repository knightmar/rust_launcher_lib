use std::error::Error;

use mc_auth::AuthFlow;
use reqwest::header::{AUTHORIZATION, HeaderMap};

use crate::auth::profile::User;

mod profile;

const CLIENT_ID: &str = "f8c516cc-122f-4701-89eb-c9bbf789028a";

pub struct Authenticator {
    access_token: String,
}

impl Authenticator {
    pub fn authenticate_ms(&self) -> Result<Authenticator, Box<dyn Error>> {
        let mut auth = AuthFlow::new(CLIENT_ID);
        let code_res = auth.request_code()?;

        println!(
            "Open this link in your browser {} and enter the following code: {}\nWaiting authentication...",
            code_res.verification_uri, code_res.user_code
        );

        auth.wait_for_login()?;
        auth.login_in_xbox_live()?;

        let minecraft = auth.login_in_minecraft()?;

        Ok(Authenticator {
            access_token: minecraft.access_token.to_string(),
        })
    }

    pub fn get_profile(&self) -> Result<User, Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", self.access_token).parse().unwrap(),
        );
        let client = reqwest::Client::new();
        let mut user: User = User::new();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            if let Ok(res) = client
                .get("https://api.minecraftservices.com/minecraft/profile")
                .headers(headers)
                .send()
                .await
            {
                if let Ok(text) = res.text().await {
                    println!("{}", text);
                    if let Ok(parse) = serde_json::from_str(&text) {
                        user = parse
                    }
                }
            }
        });

        println!("{}", user.name());

        Ok(user)
    }
    pub fn new() -> Self {
        Self {
            access_token: String::new(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AuthErrors {
    #[error("error while authentificate oauth2: `{0}`.")]
    OAuth2(String),
    #[error("error while authentificate xboxlive: `{0}`.")]
    XboxLive(String),
    #[error("error while authentificate xsts: `{0}`.")]
    Xsts(String),
    #[error("error while authentificate in minecraft: `{0}`.")]
    Minecraft(String),
}

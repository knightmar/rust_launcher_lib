#[derive(Debug)]
pub enum AuthErrors {
    OAuth2(String),
    XboxLive(String),
    Xsts(String),
    Minecraft(String),
}
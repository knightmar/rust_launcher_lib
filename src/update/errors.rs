#[derive(Debug)]
pub enum UpdateErrors {
    Fetch(String),
    Install(String),
    Download(String),
    Verification(String),
}
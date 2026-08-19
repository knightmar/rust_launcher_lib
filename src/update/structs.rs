#[derive(Clone)]
pub struct UpdateFile {
    pub url: String,
    pub name: String,
    pub hash: String,
    pub size: u64,
    pub local_path: String
}
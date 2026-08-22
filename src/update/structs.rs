#[derive(Clone, Debug)]
pub struct UpdateFile {
    pub url: String,
    pub hash: String,
    pub size: u64,
    pub local_path: String
}
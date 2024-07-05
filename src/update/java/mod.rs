use std::env;
use crate::update::java::structs::ZuluRoot;

mod structs;

// get the java runtime link to install in $BASE_DIR/runtime
pub async fn get_java_zulu_dl_link(version: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut versions = None;

    if let Ok(res) = client
        .get(format!("https://api.azul.com/metadata/v1/zulu/packages/?java_version={}&os={}&arch={}&java_package_type=jdk&javafx_bundled=false&release_status=ga", version, env::consts::OS, "x86_64"))
        .header("accept", "application/json")
        .send()
        .await
    {
        if let Ok(text) = res.text().await {
            if let Ok(parse) = serde_json::from_str::<ZuluRoot>(&text) {
                versions = Some(parse)
            }
        }
    }

    if let Some(versions) = versions {
        for version in &versions {
            if version.name.ends_with(".zip") {
                return Ok(version.download_url.clone());
            }
        }

        return Err("No versions found".into());
    }
    return Err("Error getting versions".into());
}

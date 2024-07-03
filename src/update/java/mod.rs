use crate::update::java::structs::ZuluRoot;

mod structs;

// get the java runtime link to install in $BASE_DIR/runtime
pub async fn get_java_zulu_dl_link(version: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut versions = None;
    let mut dl_link = None;

    if let Ok(res) = client
        .get(format!("https://api.azul.com/metadata/v1/zulu/packages/?java_version={}&os={}&arch={}&java_package_type=jdk&javafx_bundled=false&release_status=ga", version, "windows", "x86_64"))
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
        let version = versions.first();

        if version.is_none() {
            return Err("No versions found".into());
        } else {
            dl_link = Some(version.unwrap().download_url.clone());
        }

        println!("Download link: {}", dl_link.as_ref().unwrap());
    }

    match dl_link {
        Some(link) => Ok(link),
        None => Err("Could not find the download link".into()),
    }
}

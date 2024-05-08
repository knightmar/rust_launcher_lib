pub fn get_relative_local_dir_path(local_dir_path: &str) -> String {
    #[cfg(unix)]
    {
        let app_Root = std::env::var("HOME").expect("No HOME directory");
        app_Root + "/" + local_dir_path + "/"
    }

    #[cfg(windows)]
    {
        let app_Root = std::env::var("APPDATA").expect("No APP_Root directory");
        app_Root + "\\" + local_dir_path + "\\"
    }
}

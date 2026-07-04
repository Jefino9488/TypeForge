pub mod config {
    use directories::ProjectDirs;

    pub fn get_socket_path() -> String {
        "/tmp/typeforge.sock".to_string() // Hardcoded for MVP, in future use XDG_RUNTIME_DIR
    }

    pub fn get_learning_db_path() -> String {
        if let Some(proj_dirs) = ProjectDirs::from("com", "TypeForge", "TypeForge") {
            let data_dir = proj_dirs.data_dir();
            if !data_dir.exists() {
                let _ = std::fs::create_dir_all(data_dir);
            }
            return data_dir
                .join("learning.db")
                .to_string_lossy()
                .to_string();
        }
        "learning.db".to_string()
    }

    pub fn get_telemetry_db_path() -> String {
        if let Some(proj_dirs) = ProjectDirs::from("com", "TypeForge", "TypeForge") {
            let data_dir = proj_dirs.data_dir();
            if !data_dir.exists() {
                let _ = std::fs::create_dir_all(data_dir);
            }
            return data_dir
                .join("telemetry.db")
                .to_string_lossy()
                .to_string();
        }
        "telemetry.db".to_string()
    }
}

pub mod config {
    use directories::ProjectDirs;
    use std::path::PathBuf;

    pub fn get_socket_path() -> String {
        "/tmp/typeforge.sock".to_string() // Hardcoded for MVP, in future use XDG_RUNTIME_DIR
    }

    pub fn get_db_path() -> String {
        if let Some(proj_dirs) = ProjectDirs::from("com", "TypeForge", "TypeForge") {
            let data_dir = proj_dirs.data_dir();
            if !data_dir.exists() {
                let _ = std::fs::create_dir_all(data_dir);
            }
            return data_dir.join("typeforge_learned.db").to_string_lossy().to_string();
        }
        "typeforge_learned.db".to_string()
    }
}

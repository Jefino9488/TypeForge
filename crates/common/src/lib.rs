pub mod dict_format;
pub mod config {
    use directories::ProjectDirs;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::path::PathBuf;

    pub fn get_socket_path() -> String {
        "/tmp/typeforge.sock".to_string() // Hardcoded for MVP, in future use XDG_RUNTIME_DIR
    }

    pub fn get_learning_db_path() -> String {
        if let Some(proj_dirs) = ProjectDirs::from("com", "TypeForge", "TypeForge") {
            let data_dir = proj_dirs.data_dir();
            if !data_dir.exists() {
                let _ = std::fs::create_dir_all(data_dir);
            }
            return data_dir.join("learning.db").to_string_lossy().to_string();
        }
        "learning.db".to_string()
    }

    pub fn get_telemetry_db_path() -> String {
        if let Some(proj_dirs) = ProjectDirs::from("com", "TypeForge", "TypeForge") {
            let data_dir = proj_dirs.data_dir();
            if !data_dir.exists() {
                let _ = std::fs::create_dir_all(data_dir);
            }
            return data_dir.join("telemetry.db").to_string_lossy().to_string();
        }
        "telemetry.db".to_string()
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    #[serde(default)]
    pub struct GeneralConfig {
        pub learning: bool,
        pub candidate_limit: usize,
    }

    impl Default for GeneralConfig {
        fn default() -> Self {
            Self {
                learning: true,
                candidate_limit: 5,
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    #[serde(default)]
    pub struct DictionaryConfig {
        pub language: String,
        pub path: String,
    }

    impl Default for DictionaryConfig {
        fn default() -> Self {
            let dict_path = ProjectDirs::from("com", "typeforge", "typeforge")
                .map(|d| d.data_dir().join("dictionary.bin"))
                .unwrap_or_else(|| PathBuf::from("/usr/share/typeforge/dictionary.bin"))
                .to_string_lossy()
                .to_string();

            Self {
                language: "en".to_string(),
                path: dict_path,
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    #[serde(default)]
    pub struct LoggingConfig {
        pub level: String,
    }

    impl Default for LoggingConfig {
        fn default() -> Self {
            Self {
                level: "info".to_string(),
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    #[serde(default)]
    pub struct SessionConfig {
        pub memory: bool,
    }

    impl Default for SessionConfig {
        fn default() -> Self {
            Self { memory: true }
        }
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    #[serde(default)]
    pub struct RankingConfig {
        pub base_frequency: f32,
        pub user_frequency: f32,
        pub context: f32,
        pub session: f32,
        pub ngram: f32,
        pub candidate_limit: usize,
        pub minimum_confidence: f32,
        pub minimum_prefix: usize,
        pub enable_fuzzy: bool,
        pub enable_context: bool,
        pub enable_session: bool,
    }

    impl Default for RankingConfig {
        fn default() -> Self {
            Self {
                base_frequency: 0.35,
                user_frequency: 0.30,
                context: 0.15,
                session: 0.10,
                ngram: 0.10,
                candidate_limit: 15,
                minimum_confidence: 0.1,
                minimum_prefix: 1,
                enable_fuzzy: true,
                enable_context: true,
                enable_session: true,
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
    pub struct AppConfig {
        #[serde(default)]
        pub general: GeneralConfig,
        #[serde(default)]
        pub dictionary: DictionaryConfig,
        #[serde(default)]
        pub logging: LoggingConfig,
        #[serde(default)]
        pub session: SessionConfig,
        #[serde(default)]
        pub ranking: RankingConfig,
    }

    impl AppConfig {
        pub fn load() -> Self {
            let config_path = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("~/.config"))
                .join("typeforge")
                .join("config.toml");

            if config_path.exists()
                && let Ok(content) = fs::read_to_string(&config_path)
                && let Ok(config) = toml::from_str(&content)
            {
                return config;
            }

            AppConfig::default()
        }
    }
}

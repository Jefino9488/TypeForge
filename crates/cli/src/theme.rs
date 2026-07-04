use ini::Ini;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ThemeMetadata {
    pub name: String,
    pub author: String,
    pub version: String,
    pub supports_horizontal: bool,
    pub supports_vertical: bool,
}

pub fn get_classicui_conf_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut p| {
        p.push("fcitx5");
        p.push("conf");
        p.push("classicui.conf");
        p
    })
}

pub fn get_user_themes_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|mut p| {
        p.push("fcitx5");
        p.push("themes");
        p
    })
}

pub fn list_themes() {
    let themes_dir = match get_user_themes_dir() {
        Some(dir) => dir,
        None => {
            println!("Could not determine local data directory.");
            return;
        }
    };

    if !themes_dir.exists() {
        println!("No themes installed in {}", themes_dir.display());
        return;
    }

    println!("Available Themes:");
    if let Ok(entries) = fs::read_dir(themes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let toml_path = path.join("theme.toml");
                if toml_path.exists() {
                    if let Ok(content) = fs::read_to_string(&toml_path) {
                        if let Ok(metadata) = toml::from_str::<ThemeMetadata>(&content) {
                            let theme_id = path.file_name().unwrap().to_string_lossy();
                            println!("- {} ({}) - by {}", metadata.name, theme_id, metadata.author);
                        }
                    }
                } else {
                    let theme_id = path.file_name().unwrap().to_string_lossy();
                    println!("- {} (No metadata)", theme_id);
                }
            }
        }
    }
}

pub fn backup_conf_if_needed(conf_path: &Path) {
    let backup_path = conf_path.with_extension("conf.bak");
    if !backup_path.exists() && conf_path.exists() {
        if let Err(e) = fs::copy(conf_path, &backup_path) {
            println!("Failed to create backup: {}", e);
        } else {
            println!("Created backup of classicui.conf");
        }
    }
}

pub fn reload_fcitx() {
    println!("Reloading Fcitx5...");
    let status = Command::new("fcitx5-remote").arg("-r").status();
    if let Ok(s) = status {
        if !s.success() {
            println!("Failed to reload Fcitx5. Please restart it manually.");
        }
    } else {
         println!("Could not find fcitx5-remote. Is Fcitx5 installed?");
    }
}

pub fn apply_theme(theme_name: &str) {
    let conf_path = match get_classicui_conf_path() {
        Some(p) => p,
        None => return,
    };

    backup_conf_if_needed(&conf_path);

    let mut conf = if conf_path.exists() {
        Ini::load_from_file(&conf_path).unwrap_or_else(|_| Ini::new())
    } else {
        Ini::new()
    };

    conf.with_section(None::<String>).set("Theme", theme_name);

    if let Some(parent) = conf_path.parent() {
        fs::create_dir_all(parent).unwrap_or_default();
    }

    if let Err(e) = conf.write_to_file(&conf_path) {
        println!("Error writing config: {}", e);
    } else {
        println!("Theme set to {}", theme_name);
        reload_fcitx();
    }
}

pub fn restore_theme() {
    let conf_path = match get_classicui_conf_path() {
        Some(p) => p,
        None => return,
    };
    let backup_path = conf_path.with_extension("conf.bak");
    if backup_path.exists() {
        if let Err(e) = fs::copy(&backup_path, &conf_path) {
            println!("Failed to restore backup: {}", e);
        } else {
            println!("Restored classicui.conf from backup");
            reload_fcitx();
        }
    } else {
        println!("No backup found at {}", backup_path.display());
    }
}

pub fn current_theme() {
    let conf_path = match get_classicui_conf_path() {
        Some(p) => p,
        None => return,
    };
    if let Ok(conf) = Ini::load_from_file(&conf_path) {
        if let Some(section) = conf.section(None::<String>) {
            if let Some(theme) = section.get("Theme") {
                println!("Current Theme: {}", theme);
                return;
            }
        }
    }
    println!("Theme not set in classicui.conf");
}

pub fn doctor_info() {
    let conf_path = match get_classicui_conf_path() {
        Some(p) => p,
        None => return,
    };
    if let Ok(conf) = Ini::load_from_file(&conf_path) {
        if let Some(section) = conf.section(None::<String>) {
            let theme = section.get("Theme").unwrap_or("Default");
            let vertical = section.get("Vertical Candidate List").unwrap_or("True");
            let layout = if vertical.to_lowercase() == "false" { "Horizontal" } else { "Vertical" };
            
            println!("Theme: {}", theme);
            println!("Layout: {}", layout);
        }
    } else {
        println!("Theme: Default");
        println!("Layout: Default");
    }
}

pub fn set_layout(layout: &str) {
    let conf_path = match get_classicui_conf_path() {
        Some(p) => p,
        None => return,
    };

    backup_conf_if_needed(&conf_path);

    let mut conf = if conf_path.exists() {
        Ini::load_from_file(&conf_path).unwrap_or_else(|_| Ini::new())
    } else {
        Ini::new()
    };

    let vertical_val = match layout.to_lowercase().as_str() {
        "horizontal" => "False",
        "vertical" => "True",
        _ => {
            println!("Unknown layout. Use 'horizontal' or 'vertical'.");
            return;
        }
    };

    conf.with_section(None::<String>).set("Vertical Candidate List", vertical_val);

    if let Some(parent) = conf_path.parent() {
        fs::create_dir_all(parent).unwrap_or_default();
    }

    if let Err(e) = conf.write_to_file(&conf_path) {
        println!("Error writing config: {}", e);
    } else {
        println!("Layout set to {}", layout);
        reload_fcitx();
    }
}

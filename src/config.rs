use serde::Deserialize;
use std::{collections::HashMap, fs};

pub const FOCUS_WALLPAPER_NAME: &str = "focus_wallpaper.jpg";
const APPS_CONFIG_FILE: &str = "apps.toml";

#[derive(Debug, Deserialize)]
struct AppsConfig {
    apps: HashMap<String, Vec<String>>,
}

fn default_focus_apps() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        vec![
            r"C:\Windows\System32\notepad.exe".to_string(),
            r"C:\Users\Faizy\AppData\Local\BraveSoftware\Brave-Browser\Application\brave.exe"
                .to_string(),
        ]
    }

    #[cfg(target_os = "linux")]
    {
        vec!["brave-browser".to_string()]
    }

    #[cfg(target_os = "macos")]
    {
        vec!["TextEdit".to_string(), "Google Chrome".to_string()]
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        vec![]
    }
}

fn parse_apps_config(contents: &str) -> Result<AppsConfig, toml::de::Error> {
    toml::from_str(contents)
}

fn select_apps_for_current_os(config: &AppsConfig) -> Option<Vec<String>> {
    config.apps.get(std::env::consts::OS).cloned()
}

fn sanitize_apps(apps: Vec<String>) -> Vec<String> {
    apps.into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn load_focus_apps() -> Vec<String> {
    match fs::read_to_string(APPS_CONFIG_FILE) {
        Ok(contents) => match parse_apps_config(&contents) {
            Ok(config) => {
                let selected = select_apps_for_current_os(&config).unwrap_or_default();
                let cleaned = sanitize_apps(selected);
                if cleaned.is_empty() {
                    let defaults = default_focus_apps();
                    println!(
                        "No apps configured for OS '{}' in '{}'. Using {} default app(s).",
                        std::env::consts::OS,
                        APPS_CONFIG_FILE,
                        defaults.len()
                    );
                    defaults
                } else {
                    println!(
                        "Loaded {} app(s) for OS '{}' from '{}'.",
                        cleaned.len(),
                        std::env::consts::OS,
                        APPS_CONFIG_FILE
                    );
                    cleaned
                }
            }
            Err(e) => {
                eprintln!(
                    "ERROR: Could not parse '{}': {}. Falling back to defaults.",
                    APPS_CONFIG_FILE, e
                );
                default_focus_apps()
            }
        },
        Err(e) => {
            eprintln!(
                "ERROR: Could not read '{}': {}. Falling back to defaults.",
                APPS_CONFIG_FILE, e
            );
            default_focus_apps()
        }
    }
}

use crate::config::FOCUS_WALLPAPER_NAME;
use std::fs;
#[cfg(target_os = "windows")]
use std::path::Path;
use std::process::Command;
use std::sync::{LazyLock, Mutex};

static ORIGINAL_WALLPAPER_PATH: LazyLock<Mutex<Option<String>>> =
    LazyLock::new(|| Mutex::new(None));

fn launch_app(app: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").args(["-a", app]).spawn().map(|_| ())
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        Command::new(app).spawn().map(|_| ())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err(std::io::Error::other(format!(
            "Unsupported OS: {}",
            std::env::consts::OS
        )))
    }
}

fn terminate_app(app: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let image_name = Path::new(app)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(app);

        let output = Command::new("taskkill")
            .args(["/F", "/IM", image_name])
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(std::io::Error::other(format!(
                "taskkill failed for '{}': {}",
                image_name,
                stderr.trim()
            )))
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let output = Command::new("pkill").args(["-f", app]).output()?;

        // pkill exit code: 0 => matched/killed, 1 => no matching process (safe for us)
        if output.status.success() || output.status.code() == Some(1) {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(std::io::Error::other(format!(
                "pkill failed for '{}': {}",
                app,
                stderr.trim()
            )))
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err(std::io::Error::other(format!(
            "Unsupported OS: {}",
            std::env::consts::OS
        )))
    }
}

#[cfg(target_os = "linux")]
fn gsettings_get_wallpaper_uri() -> Option<String> {
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.background", "picture-uri"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let trimmed = raw.trim_matches('\'').trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

#[cfg(target_os = "linux")]
fn gsettings_set_wallpaper_uri(uri: &str) -> bool {
    let status = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-uri", uri])
        .status();

    match status {
        Ok(s) if s.success() => {
            // Try dark-variant key as well (GNOME 42+ / some distros), ignore failure.
            let _ = Command::new("gsettings")
                .args([
                    "set",
                    "org.gnome.desktop.background",
                    "picture-uri-dark",
                    uri,
                ])
                .status();
            true
        }
        _ => false,
    }
}

#[cfg(target_os = "linux")]
fn path_to_file_uri(path: &str) -> String {
    let normalized = path.replace(' ', "%20");
    if normalized.starts_with("file://") {
        normalized
    } else {
        format!("file://{}", normalized)
    }
}

fn save_original_wallpaper_path() {
    if let Ok(path) = wallpaper::get() {
        println!("Saved original wallpaper: {}", path);
        *ORIGINAL_WALLPAPER_PATH.lock().unwrap() = Some(path);
        return;
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(uri) = gsettings_get_wallpaper_uri() {
            println!("Saved original wallpaper from gsettings: {}", uri);
            *ORIGINAL_WALLPAPER_PATH.lock().unwrap() = Some(uri);
        }
    }
}

fn set_focus_wallpaper() {
    match fs::canonicalize(FOCUS_WALLPAPER_NAME) {
        Ok(absolute_path) => {
            let path_str = match absolute_path.to_str() {
                Some(p) => p,
                None => {
                    eprintln!("ERROR: Focus wallpaper path contains invalid UTF-8.");
                    return;
                }
            };

            #[cfg(target_os = "linux")]
            {
                let uri = path_to_file_uri(path_str);
                if gsettings_set_wallpaper_uri(&uri) {
                    println!("Focus wallpaper has been set (gsettings primary).");
                    return;
                }

                if wallpaper::set_from_path(path_str).is_ok() {
                    println!("Focus wallpaper has been set (wallpaper crate fallback).");
                    return;
                }

                eprintln!(
                    "ERROR: Failed to set focus wallpaper using gsettings and wallpaper crate."
                );
            }

            #[cfg(not(target_os = "linux"))]
            {
                if wallpaper::set_from_path(path_str).is_ok() {
                    println!("Focus wallpaper has been set (wallpaper crate).");
                    return;
                }

                eprintln!("ERROR: Failed to set focus wallpaper using wallpaper crate.");
            }
        }
        Err(e) => eprintln!(
            "ERROR: Could not find wallpaper '{}': {}",
            FOCUS_WALLPAPER_NAME, e
        ),
    }
}

fn restore_original_wallpaper() {
    let mut original_path = ORIGINAL_WALLPAPER_PATH.lock().unwrap();
    if let Some(path) = original_path.as_deref() {
        #[cfg(target_os = "linux")]
        {
            let uri = if path.starts_with("file://") {
                path.to_string()
            } else {
                path_to_file_uri(path)
            };

            if gsettings_set_wallpaper_uri(&uri) {
                println!("Restored original wallpaper (gsettings primary): {}", uri);
                *original_path = None;
                return;
            }

            if wallpaper::set_from_path(path).is_ok() {
                println!(
                    "Restored original wallpaper (wallpaper crate fallback): {}",
                    path
                );
                *original_path = None;
                return;
            }

            eprintln!(
                "ERROR: Failed to restore original wallpaper using gsettings and wallpaper crate."
            );
            *original_path = None;
            return;
        }

        #[cfg(not(target_os = "linux"))]
        {
            if wallpaper::set_from_path(path).is_ok() {
                println!("Restored original wallpaper (wallpaper crate): {}", path);
                *original_path = None;
                return;
            }

            eprintln!("ERROR: Failed to restore original wallpaper using wallpaper crate.");
        }
    }

    *original_path = None;
}

pub fn activate_focus_mode(focus_apps: &[String]) {
    println!("Activating focus mode automations...");

    // Save current wallpaper so we can restore it later
    save_original_wallpaper_path();

    // Set focus wallpaper (with Linux GNOME fallback)
    set_focus_wallpaper();

    // Launch apps
    println!(
        "Launching focus applications for {}...",
        std::env::consts::OS
    );
    for app in focus_apps {
        match launch_app(app) {
            Ok(_) => println!("Successfully launched '{}'", app),
            Err(e) => eprintln!("ERROR: Failed to launch '{}': {}", app, e),
        }
    }
}

pub fn deactivate_focus_mode(focus_apps: &[String]) {
    println!("Deactivating focus mode automations...");

    // Restore wallpaper (with Linux GNOME fallback)
    restore_original_wallpaper();

    // Close apps
    println!("Closing focus applications for {}...", std::env::consts::OS);
    for app in focus_apps {
        if let Err(e) = terminate_app(app) {
            eprintln!("ERROR: Failed to close '{}': {}", app, e);
        }
    }
}

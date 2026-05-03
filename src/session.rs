use chrono::{DateTime, Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::OnceLock,
};

static FIRST_WRITE_PATH_LOGGED: OnceLock<()> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRecord {
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub duration_minutes: f32,
    pub hour_of_day: u8,
    pub day_of_week: u8,
    pub interrupted: bool,
}

impl SessionRecord {
    pub fn new(start_time: DateTime<Local>, end_time: DateTime<Local>) -> Self {
        let duration = end_time.signed_duration_since(start_time);
        let duration_minutes = (duration.num_seconds().max(0) as f32) / 60.0;
        let hour_of_day = start_time.hour() as u8;
        let day_of_week = start_time.weekday().num_days_from_monday() as u8;
        let interrupted = duration_minutes < 10.0;

        Self {
            start_time,
            end_time,
            duration_minutes,
            hour_of_day,
            day_of_week,
            interrupted,
        }
    }
}

pub fn sessions_file_path() -> PathBuf {
    if let Some(mut data_dir) = dirs::data_dir() {
        #[cfg(target_os = "windows")]
        data_dir.push("FocusTotem");

        #[cfg(not(target_os = "windows"))]
        data_dir.push("focus_totem");

        data_dir.push("sessions.json");
        return data_dir;
    }

    PathBuf::from("sessions.json")
}

pub fn load_sessions() -> io::Result<Vec<SessionRecord>> {
    read_sessions(&sessions_file_path())
}

pub fn append_session(record: SessionRecord) -> io::Result<PathBuf> {
    let path = sessions_file_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut sessions = read_sessions(&path)?;
    sessions.push(record);
    write_sessions_atomically(&path, &sessions)?;

    FIRST_WRITE_PATH_LOGGED.get_or_init(|| {
        println!("Session log file: {}", path.display());
    });

    Ok(path)
}

fn read_sessions(path: &Path) -> io::Result<Vec<SessionRecord>> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            if contents.trim().is_empty() {
                return Ok(Vec::new());
            }

            serde_json::from_str(&contents).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse existing sessions JSON: {e}"),
                )
            })
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

fn write_sessions_atomically(path: &Path, sessions: &[SessionRecord]) -> io::Result<()> {
    let tmp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(sessions).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize sessions JSON: {e}"),
        )
    })?;

    fs::write(&tmp_path, json)?;

    // On Unix-like systems, renaming over an existing file is atomic.
    // On Windows, std::fs::rename fails if the destination exists, so remove first.
    #[cfg(target_os = "windows")]
    {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }

    fs::rename(tmp_path, path)?;

    Ok(())
}

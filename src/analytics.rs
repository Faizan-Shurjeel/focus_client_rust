use crate::session::SessionRecord;
use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveDate};
use linfa::prelude::*;
use linfa_linear::LinearRegression;
use linfa_trees::{DecisionTree, SplitQuality};
use ndarray::{Array1, Array2};
use std::collections::BTreeMap;

const TREND_MIN_DAYS: usize = 7;
const TREE_MIN_SESSIONS: usize = 30;

#[derive(Debug)]
pub struct AnalyticsResult {
    // Layer 1 — always present
    pub total_sessions: usize,
    pub distraction_rate: f32,
    pub top_focus_hours: Vec<(u8, f32)>,
    pub best_days: Vec<(u8, f32)>,
    pub weekly_total_minutes: f32,
    pub weekly_delta_minutes: f32,

    // Layer 2 — None if < 7 calendar days of data
    pub trend_slope: Option<f32>,
    pub trend_label: Option<String>,

    // Layer 3 — None if < 30 sessions
    pub tree_rules: Option<String>,
    pub quality_rate: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionQuality {
    Quality = 0,
    Shallow = 1,
    Distracted = 2,
}

impl SessionQuality {
    fn as_label(self) -> usize {
        self as usize
    }

    fn from_label(label: usize) -> Self {
        match label {
            0 => Self::Quality,
            1 => Self::Shallow,
            _ => Self::Distracted,
        }
    }
}

fn label_session(session: &SessionRecord) -> SessionQuality {
    if session.duration_minutes >= 20.0 && !session.interrupted {
        SessionQuality::Quality
    } else if session.duration_minutes >= 10.0 {
        SessionQuality::Shallow
    } else {
        SessionQuality::Distracted
    }
}

pub fn run_analytics(sessions: &[SessionRecord]) -> AnalyticsResult {
    let total_sessions = sessions.len();

    if sessions.is_empty() {
        return AnalyticsResult {
            total_sessions,
            distraction_rate: 0.0,
            top_focus_hours: Vec::new(),
            best_days: Vec::new(),
            weekly_total_minutes: 0.0,
            weekly_delta_minutes: 0.0,
            trend_slope: None,
            trend_label: None,
            tree_rules: None,
            quality_rate: None,
        };
    }

    let distraction_rate = distraction_rate(sessions);
    let top_focus_hours = top_focus_hours(sessions);
    let best_days = best_days(sessions);
    let (weekly_total_minutes, weekly_delta_minutes) = weekly_summary(sessions);
    let daily_totals = daily_totals(sessions);
    let (trend_slope, trend_label) = trend_detection(&daily_totals)
        .map(|(slope, label)| (Some(slope), Some(label)))
        .unwrap_or((None, None));
    let (tree_rules, quality_rate) = decision_tree_summary(sessions)
        .map(|(rules, rate)| (Some(rules), Some(rate)))
        .unwrap_or((None, None));

    AnalyticsResult {
        total_sessions,
        distraction_rate,
        top_focus_hours,
        best_days,
        weekly_total_minutes,
        weekly_delta_minutes,
        trend_slope,
        trend_label,
        tree_rules,
        quality_rate,
    }
}

pub fn format_analytics(result: &AnalyticsResult) -> String {
    let mut lines = Vec::new();

    lines.push("Focus Analytics".to_string());
    lines.push("===============".to_string());
    lines.push(format!("Total sessions: {}", result.total_sessions));
    lines.push(format!(
        "Distraction rate: {:.1}%{}",
        result.distraction_rate,
        if result.distraction_rate > 30.0 {
            " — high; protect your focus blocks"
        } else {
            ""
        }
    ));
    lines.push(format!(
        "This week: {} ({})",
        format_minutes(result.weekly_total_minutes),
        format_delta_minutes(result.weekly_delta_minutes)
    ));

    lines.push(String::new());
    lines.push("Top focus hours:".to_string());
    if result.top_focus_hours.is_empty() {
        lines.push("  Not enough completed sessions yet.".to_string());
    } else {
        for (hour, score) in &result.top_focus_hours {
            lines.push(format!(
                "  {}:00–{}:00 (score: {:.1})",
                hour,
                (*hour + 1) % 24,
                score
            ));
        }
    }

    lines.push(String::new());
    lines.push("Best days:".to_string());
    if result.best_days.is_empty() {
        lines.push("  Not enough completed sessions yet.".to_string());
    } else {
        for (day, mean_duration) in &result.best_days {
            lines.push(format!(
                "  {} (mean: {})",
                day_name(*day),
                format_minutes(*mean_duration)
            ));
        }
    }

    lines.push(String::new());
    lines.push("Trend model:".to_string());
    match (&result.trend_slope, &result.trend_label) {
        (Some(slope), Some(label)) => {
            lines.push(format!("  {} (slope: {:.2} min/day)", label, slope));
        }
        _ => {
            lines.push("  Needs at least 7 calendar days of data — collecting data.".to_string());
        }
    }

    lines.push(String::new());
    lines.push("Decision tree quality model:".to_string());
    match (&result.tree_rules, &result.quality_rate) {
        (Some(rules), Some(rate)) => {
            lines.push(format!("  Quality rate: {:.1}%", rate));
            for line in rules.lines() {
                lines.push(format!("  {}", line));
            }
        }
        _ => {
            let remaining = TREE_MIN_SESSIONS.saturating_sub(result.total_sessions);
            lines.push(format!(
                "  Predictive model needs {} more session(s) — collecting data.",
                remaining
            ));
        }
    }

    lines.join("\n")
}

fn distraction_rate(sessions: &[SessionRecord]) -> f32 {
    if sessions.is_empty() {
        return 0.0;
    }

    let interrupted = sessions.iter().filter(|s| s.interrupted).count();
    (interrupted as f32 / sessions.len() as f32) * 100.0
}

fn top_focus_hours(sessions: &[SessionRecord]) -> Vec<(u8, f32)> {
    let mut totals = [0.0_f32; 24];
    let mut counts = [0_u32; 24];

    for session in sessions {
        let hour = session.hour_of_day.min(23) as usize;
        totals[hour] += session.duration_minutes;
        counts[hour] += 1;
    }

    let mut scored: Vec<(u8, f32)> = (0..24)
        .filter(|hour| counts[*hour] > 0)
        .map(|hour| {
            let mean = totals[hour] / counts[hour] as f32;
            let score = mean * counts[hour] as f32;
            (hour as u8, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored.truncate(3);
    scored
}

fn best_days(sessions: &[SessionRecord]) -> Vec<(u8, f32)> {
    let mut totals = [0.0_f32; 7];
    let mut counts = [0_u32; 7];

    for session in sessions {
        let day = session.day_of_week.min(6) as usize;
        totals[day] += session.duration_minutes;
        counts[day] += 1;
    }

    let mut scored: Vec<(u8, f32)> = (0..7)
        .filter(|day| counts[*day] > 0)
        .map(|day| (day as u8, totals[day] / counts[day] as f32))
        .collect();

    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored.truncate(2);
    scored
}

fn weekly_summary(sessions: &[SessionRecord]) -> (f32, f32) {
    let now = Local::now();
    let current_week_start_date =
        now.date_naive() - ChronoDuration::days(now.weekday().num_days_from_monday() as i64);
    let current_week_start = current_week_start_date
        .and_hms_opt(0, 0, 0)
        .expect("valid current week start");
    let previous_week_start = current_week_start - ChronoDuration::days(7);

    let mut current_week_total = 0.0;
    let mut previous_week_total = 0.0;

    for session in sessions {
        let start = session.start_time.naive_local();
        if start >= current_week_start {
            current_week_total += session.duration_minutes;
        } else if start >= previous_week_start && start < current_week_start {
            previous_week_total += session.duration_minutes;
        }
    }

    (current_week_total, current_week_total - previous_week_total)
}

fn daily_totals(sessions: &[SessionRecord]) -> BTreeMap<NaiveDate, f32> {
    let mut totals = BTreeMap::new();
    for session in sessions {
        *totals.entry(session.start_time.date_naive()).or_insert(0.0) += session.duration_minutes;
    }
    totals
}

fn trend_detection(daily_totals: &BTreeMap<NaiveDate, f32>) -> Option<(f32, String)> {
    let n_days = daily_totals.len();
    if n_days < TREND_MIN_DAYS {
        return None;
    }

    let day_indices: Vec<f64> = (0..n_days).map(|i| i as f64).collect();
    let totals: Vec<f64> = daily_totals
        .values()
        .map(|minutes| *minutes as f64)
        .collect();

    let x = Array2::from_shape_vec((n_days, 1), day_indices).ok()?;
    let y = Array1::from_vec(totals);
    let dataset = linfa::Dataset::new(x, y);
    let model = LinearRegression::default().fit(&dataset).ok()?;
    let slope = model.params()[0] as f32;

    let label = if slope > 0.5 {
        "📈 Focus trending up".to_string()
    } else if slope < -0.5 {
        "📉 Focus declining — protect your blocks".to_string()
    } else {
        "Focus is steady".to_string()
    };

    Some((slope, label))
}

fn decision_tree_summary(sessions: &[SessionRecord]) -> Option<(String, f32)> {
    if sessions.len() < TREE_MIN_SESSIONS {
        return None;
    }

    let feature_values: Vec<f64> = sessions
        .iter()
        .flat_map(|session| [session.hour_of_day as f64, session.day_of_week as f64])
        .collect();
    let label_values: Vec<usize> = sessions
        .iter()
        .map(|session| label_session(session).as_label())
        .collect();

    let features = Array2::from_shape_vec((sessions.len(), 2), feature_values).ok()?;
    let labels = Array1::from_vec(label_values.clone());
    let dataset = linfa::Dataset::new(features, labels);

    let model = DecisionTree::params()
        .split_quality(SplitQuality::Gini)
        .max_depth(Some(4))
        .min_weight_split(5.0)
        .fit(&dataset)
        .ok()?;

    let predictions = model.predict(&dataset);
    let correct = predictions
        .iter()
        .zip(label_values.iter())
        .filter(|(predicted, actual)| **predicted == **actual)
        .count();
    let training_accuracy = (correct as f32 / sessions.len() as f32) * 100.0;

    let quality_count = sessions
        .iter()
        .filter(|session| label_session(session) == SessionQuality::Quality)
        .count();
    let quality_rate = (quality_count as f32 / sessions.len() as f32) * 100.0;
    let predicted_quality_windows = predicted_quality_windows(&model);

    let rules = if predicted_quality_windows.is_empty() {
        format!(
            "Decision tree trained (Gini, max_depth=4, min_samples_split=5).\nTraining accuracy: {:.1}%.\nNo hour/day window is currently predicted as Quality.",
            training_accuracy
        )
    } else {
        format!(
            "Decision tree trained (Gini, max_depth=4, min_samples_split=5).\nTraining accuracy: {:.1}%.\nPredicted quality windows: {}",
            training_accuracy,
            predicted_quality_windows.join(", ")
        )
    };

    Some((rules, quality_rate))
}

fn predicted_quality_windows(model: &DecisionTree<f64, usize>) -> Vec<String> {
    let mut windows = Vec::new();

    for day in 0..7 {
        for hour in 0..24 {
            let query = match Array2::from_shape_vec((1, 2), vec![hour as f64, day as f64]) {
                Ok(query) => query,
                Err(_) => continue,
            };
            let prediction = model.predict(&query);
            let predicted_quality = prediction
                .get(0)
                .map(|label| SessionQuality::from_label(*label))
                == Some(SessionQuality::Quality);

            if predicted_quality {
                windows.push(format!("{} {}:00", day_name(day), hour));
            }
        }
    }

    windows.truncate(12);
    windows
}

fn day_name(day: u8) -> &'static str {
    match day {
        0 => "Mon",
        1 => "Tue",
        2 => "Wed",
        3 => "Thu",
        4 => "Fri",
        5 => "Sat",
        6 => "Sun",
        _ => "Unknown",
    }
}

fn format_minutes(minutes: f32) -> String {
    if minutes > 0.0 && minutes < 1.0 {
        return "<1m".to_string();
    }

    let rounded = minutes.round() as i32;
    let hours = rounded / 60;
    let mins = rounded.abs() % 60;

    if hours == 0 {
        format!("{}m", rounded)
    } else {
        format!("{}h {}m", hours, mins)
    }
}

fn format_delta_minutes(minutes: f32) -> String {
    if minutes >= 0.0 {
        format!("up {} vs last week", format_minutes(minutes))
    } else {
        format!("down {} vs last week", format_minutes(minutes.abs()))
    }
}

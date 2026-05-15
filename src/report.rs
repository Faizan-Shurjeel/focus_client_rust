use crate::analytics::AnalyticsResult;
use std::{fs, io, path::PathBuf, process::Command};

pub fn generate_report(result: &AnalyticsResult) -> String {
    let top_focus_windows = render_top_focus_windows(result);
    let trend_chart = render_trend_chart(result);
    let decision_tree = render_decision_tree(result);
    let recommendation = html_escape(&ai_recommendation(result));
    let trend_label = html_escape(
        result
            .trend_label
            .as_deref()
            .unwrap_or("Collecting enough daily history for trend detection"),
    );
    let trend_slope = result
        .trend_slope
        .map(|slope| format!("{slope:.2} min/day"))
        .unwrap_or_else(|| "needs 7 calendar days".to_string());

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Focus Health Report</title>
  <style>
    :root {{
      --bg-void: #0b0e12;
      --surface-low: #1D2024;
      --surface-highest: #2B2D31;
      --tertiary-muted: #d1f3dc;
      --primary-accent: #bbdaff;
      --text-primary: #f6f8fb;
      --text-secondary: #b6beca;
      --text-muted: #7f8794;
      --on-primary: #07111d;
      --danger-soft: #ffcfcc;
    }}

    * {{ box-sizing: border-box; }}

    body {{
      margin: 0;
      min-height: 100vh;
      background: var(--bg-void);
      color: var(--text-primary);
      font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }}

    main {{
      width: min(1120px, calc(100vw - 32px));
      margin: 0 auto;
      padding: 56px 0 72px;
    }}

    .hero {{
      background: var(--surface-low);
      border-radius: 32px;
      padding: clamp(28px, 5vw, 56px);
      margin-bottom: 24px;
    }}

    .eyebrow {{
      color: var(--primary-accent);
      font-size: 0.78rem;
      font-weight: 700;
      letter-spacing: 0.18em;
      text-transform: uppercase;
      margin: 0 0 16px;
    }}

    h1, h2, h3 {{
      font-family: Epilogue, Inter, sans-serif;
      letter-spacing: -0.04em;
      margin: 0;
    }}

    h1 {{
      font-size: clamp(2.6rem, 8vw, 5.5rem);
      line-height: 0.9;
      max-width: 850px;
    }}

    h2 {{
      font-size: clamp(1.6rem, 4vw, 2.4rem);
      margin-bottom: 18px;
    }}

    h3 {{
      font-size: 1rem;
      letter-spacing: -0.02em;
    }}

    .subhead {{
      color: var(--text-secondary);
      max-width: 680px;
      margin: 24px 0 0;
      font-size: 1.05rem;
      line-height: 1.7;
    }}

    .grid {{
      display: grid;
      grid-template-columns: repeat(12, 1fr);
      gap: 24px;
    }}

    .card {{
      background: var(--surface-low);
      border-radius: 24px;
      padding: 24px;
    }}

    .card.elevated,
    .metric,
    .list-item,
    .bar-row,
    pre {{
      background: var(--surface-highest);
      border-radius: 18px;
    }}

    .span-12 {{ grid-column: span 12; }}
    .span-7 {{ grid-column: span 7; }}
    .span-5 {{ grid-column: span 5; }}
    .span-4 {{ grid-column: span 4; }}

    .metrics {{
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 16px;
    }}

    .metric {{
      min-height: 118px;
      padding: 20px;
      display: flex;
      flex-direction: column;
      justify-content: space-between;
    }}

    .metric .label {{
      color: var(--text-muted);
      font-weight: 700;
      font-size: 0.76rem;
      letter-spacing: 0.12em;
      text-transform: uppercase;
    }}

    .metric .value {{
      color: var(--text-primary);
      font-family: Epilogue, Inter, sans-serif;
      font-size: 2rem;
      font-weight: 800;
      letter-spacing: -0.05em;
    }}

    .metric .context {{
      color: var(--text-secondary);
      font-size: 0.92rem;
    }}

    .list {{
      display: flex;
      flex-direction: column;
      gap: 16px;
    }}

    .list-item {{
      min-height: 64px;
      padding: 18px 20px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
    }}

    .rank {{
      width: 38px;
      height: 38px;
      border-radius: 999px;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      background: var(--primary-accent);
      color: var(--on-primary);
      font-weight: 800;
      margin-right: 14px;
    }}

    .muted {{ color: var(--text-muted); }}
    .accent {{ color: var(--primary-accent); }}
    .danger {{ color: var(--danger-soft); }}

    .bar-row {{
      min-height: 56px;
      padding: 14px 16px;
      display: grid;
      grid-template-columns: 82px 1fr 72px;
      align-items: center;
      gap: 14px;
    }}

    .bar-track {{
      height: 18px;
      border-radius: 999px;
      background: var(--surface-low);
      overflow: hidden;
    }}

    .bar-fill {{
      min-width: 0;
      height: 100%;
      border-radius: 999px;
      background: var(--primary-accent);
    }}

    pre {{
      color: var(--text-secondary);
      padding: 20px;
      white-space: pre-wrap;
      overflow-x: auto;
      line-height: 1.6;
      font-size: 0.94rem;
      margin: 0;
    }}

    .recommendation {{
      background: var(--tertiary-muted);
      color: #07110b;
      border-radius: 24px;
      padding: 24px;
      font-weight: 600;
      line-height: 1.7;
    }}

    @media (max-width: 860px) {{
      .grid, .metrics {{ display: flex; flex-direction: column; }}
      .span-12, .span-7, .span-5, .span-4 {{ grid-column: auto; }}
      .bar-row {{ grid-template-columns: 68px 1fr 58px; }}
    }}
  </style>
</head>
<body>
  <main>
    <section class="hero">
      <p class="eyebrow">Focus Totem Intelligence</p>
      <h1>Focus Health Report</h1>
      <p class="subhead">A local, privacy-preserving report generated from your completed focus sessions. No cloud service, no JavaScript charting library, just your session log converted into actionable signals.</p>
    </section>

    <section class="grid">
      <article class="card span-12">
        <h2>Weekly Summary</h2>
        <div class="metrics">
          <div class="metric">
            <span class="label">Focused this week</span>
            <span class="value">{weekly_hours}</span>
            <span class="context">{weekly_minutes} total</span>
          </div>
          <div class="metric">
            <span class="label">Sessions this week</span>
            <span class="value">{weekly_sessions}</span>
            <span class="context">{total_sessions} all-time logged</span>
          </div>
          <div class="metric">
            <span class="label">Delta vs last week</span>
            <span class="value">{weekly_delta}</span>
            <span class="context">distraction rate: {distraction_rate:.1}%</span>
          </div>
        </div>
      </article>

      <article class="card span-7">
        <h2>Top Focus Windows</h2>
        <div class="list">{top_focus_windows}</div>
      </article>

      <article class="card elevated span-5">
        <h2>Trend</h2>
        <p class="muted">{trend_label} <span class="accent">({trend_slope})</span></p>
        <div class="list">{trend_chart}</div>
      </article>

      <article class="card span-7">
        <h2>Decision Tree Rules</h2>
        {decision_tree}
      </article>

      <article class="card span-5">
        <h2>AI Recommendation</h2>
        <div class="recommendation">{recommendation}</div>
      </article>
    </section>
  </main>
</body>
</html>"#,
        weekly_hours = format_hours(result.weekly_total_minutes),
        weekly_minutes = format_minutes(result.weekly_total_minutes),
        weekly_sessions = result.weekly_session_count,
        total_sessions = result.total_sessions,
        weekly_delta = format_signed_delta(result.weekly_delta_minutes),
        distraction_rate = result.distraction_rate,
        top_focus_windows = top_focus_windows,
        trend_label = trend_label,
        trend_slope = html_escape(&trend_slope),
        trend_chart = trend_chart,
        decision_tree = decision_tree,
        recommendation = recommendation,
    )
}

pub fn open_in_browser(html: &str) -> io::Result<PathBuf> {
    let path = std::env::temp_dir().join("focus_report.html");
    fs::write(&path, html)?;
    open_path(&path)?;
    Ok(path)
}

fn open_path(path: &PathBuf) -> io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).spawn().map(|_| ())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(path).spawn().map(|_| ())
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn().map(|_| ())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(io::Error::other(format!(
            "Opening reports is unsupported on {}",
            std::env::consts::OS
        )))
    }
}

fn render_top_focus_windows(result: &AnalyticsResult) -> String {
    if result.top_focus_hours.is_empty() {
        return "<div class=\"list-item\"><span class=\"muted\">Complete a few focus sessions to rank your strongest hours.</span></div>".to_string();
    }

    result
        .top_focus_hours
        .iter()
        .enumerate()
        .map(|(index, (hour, score))| {
            format!(
                "<div class=\"list-item\"><div><span class=\"rank\">#{rank}</span><strong>{start}:00–{end}:00</strong></div><span class=\"muted\">score {score:.1}</span></div>",
                rank = index + 1,
                start = hour,
                end = (*hour + 1) % 24,
                score = score
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn render_trend_chart(result: &AnalyticsResult) -> String {
    if result.last_7_day_totals.is_empty() {
        return "<div class=\"bar-row\"><span class=\"muted\">No data</span><div class=\"bar-track\"><div class=\"bar-fill\" style=\"width: 0%\"></div></div><span class=\"muted\">0m</span></div>".to_string();
    }

    let max_minutes = result
        .last_7_day_totals
        .iter()
        .map(|(_, minutes)| *minutes)
        .fold(0.0_f32, f32::max);

    result
        .last_7_day_totals
        .iter()
        .map(|(date, minutes)| {
            let width = if max_minutes <= 0.0 || *minutes <= 0.0 {
                0.0
            } else {
                ((*minutes / max_minutes) * 100.0).clamp(3.0, 100.0)
            };
            format!(
                "<div class=\"bar-row\"><span class=\"muted\">{date}</span><div class=\"bar-track\"><div class=\"bar-fill\" style=\"width: {width:.1}%\"></div></div><span>{minutes}</span></div>",
                date = html_escape(&date.format("%a %d").to_string()),
                width = width,
                minutes = format_minutes(*minutes)
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn render_decision_tree(result: &AnalyticsResult) -> String {
    match (&result.tree_rules, &result.quality_rate) {
        (Some(rules), Some(rate)) => format!(
            "<p class=\"muted\">Quality rate: <span class=\"accent\">{rate:.1}%</span></p><pre>{rules}</pre>",
            rate = rate,
            rules = html_escape(rules)
        ),
        _ => {
            let remaining = 30usize.saturating_sub(result.total_sessions);
            format!(
                "<pre>Predictive model is collecting data. It needs {remaining} more completed session(s) before the decision tree activates.</pre>",
                remaining = remaining
            )
        }
    }
}

fn ai_recommendation(result: &AnalyticsResult) -> String {
    if result.total_sessions == 0 {
        return "Start by completing one focus session. Once sessions are logged, this report will identify your best hours, weekly trajectory, and predicted quality windows.".to_string();
    }

    let mut parts = Vec::new();

    if let Some((hour, _)) = result.top_focus_hours.first() {
        parts.push(format!(
            "Your strongest current focus window is {}:00–{}:00; protect that slot for deep work first.",
            hour,
            (*hour + 1) % 24
        ));
    } else {
        parts.push("Keep logging sessions so your best focus windows can stabilize.".to_string());
    }

    if result.distraction_rate > 30.0 {
        parts.push("Your interruption rate is high, so shorten your next block slightly and remove one obvious trigger before starting.".to_string());
    } else {
        parts.push("Your interruption rate is under control; keep the ritual consistent and increase session length gradually.".to_string());
    }

    match result.trend_slope {
        Some(slope) if slope > 0.5 => parts.push("The trend model is positive, which means your total focus time is compounding.".to_string()),
        Some(slope) if slope < -0.5 => parts.push("The trend model is declining; schedule a recovery block early in the week instead of waiting for motivation.".to_string()),
        Some(_) => parts.push("Your trend is steady; the next improvement should come from consistency rather than intensity.".to_string()),
        None => parts.push("Trend detection will unlock after seven calendar days of history.".to_string()),
    }

    if let Some(rate) = result.quality_rate {
        if rate >= 60.0 {
            parts.push(
                "The quality model is confident enough to keep prioritizing your existing routine."
                    .to_string(),
            );
        } else {
            parts.push("The quality model sees room to improve: favor your top-ranked hours and avoid scattered micro-sessions.".to_string());
        }
    }

    parts.join(" ")
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn format_hours(minutes: f32) -> String {
    format!("{:.1}h", minutes.max(0.0) / 60.0)
}

fn format_minutes(minutes: f32) -> String {
    if minutes > 0.0 && minutes < 1.0 {
        return "&lt;1m".to_string();
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

fn format_signed_delta(minutes: f32) -> String {
    if minutes >= 0.0 {
        format!("+{}", format_minutes(minutes))
    } else {
        format!("-{}", format_minutes(minutes.abs()))
    }
}

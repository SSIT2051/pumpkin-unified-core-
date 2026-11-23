use anyhow::Result;
use chrono::Duration;
use regex::Regex;
use std::time::Duration as StdDuration;

lazy_static::lazy_static! {
    static ref DURATION_REGEX: Regex = Regex::new(
        r"(?:(\d+)d)?(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?"
    ).unwrap();
}

pub fn parse_duration(input: &str) -> Result<Duration> {
    let input = input.trim().to_lowercase();

    if input.is_empty() {
        return Err(anyhow::anyhow!("Empty duration string"));
    }

    let caps = DURATION_REGEX
        .captures(&input)
        .ok_or_else(|| anyhow::anyhow!("Invalid duration format: {}", input))?;

    let days = caps
        .get(1)
        .and_then(|m| m.as_str().parse::<i64>().ok())
        .unwrap_or(0);
    let hours = caps
        .get(2)
        .and_then(|m| m.as_str().parse::<i64>().ok())
        .unwrap_or(0);
    let minutes = caps
        .get(3)
        .and_then(|m| m.as_str().parse::<i64>().ok())
        .unwrap_or(0);
    let seconds = caps
        .get(4)
        .and_then(|m| m.as_str().parse::<i64>().ok())
        .unwrap_or(0);

    if days == 0 && hours == 0 && minutes == 0 && seconds == 0 {
        return Err(anyhow::anyhow!("Duration must be greater than 0"));
    }

    Ok(Duration::days(days)
        + Duration::hours(hours)
        + Duration::minutes(minutes)
        + Duration::seconds(seconds))
}

pub fn format_duration(duration: Duration) -> String {
    let mut parts = Vec::new();
    let total_seconds = duration.num_seconds();

    let days = total_seconds / 86400;
    if days > 0 {
        parts.push(format!("{}d", days));
    }

    let hours = (total_seconds % 86400) / 3600;
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }

    let minutes = (total_seconds % 3600) / 60;
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }

    let seconds = total_seconds % 60;
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{}s", seconds));
    }

    parts.join(" ")
}

pub fn parse_duration_std(input: &str) -> Result<StdDuration> {
    parse_duration(input).map(|d| StdDuration::from_secs(d.num_seconds() as u64))
}

pub fn now_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

pub fn is_expired(timestamp: i64, duration_seconds: u64) -> bool {
    let now = now_timestamp();
    now > timestamp + duration_seconds as i64
}

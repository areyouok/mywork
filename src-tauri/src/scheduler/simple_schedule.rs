//! Simple schedule parser - converts JSON schedule definitions to cron expressions
//!
//! Supported formats:
//! - interval: {"type": "interval", "value": 5, "unit": "minutes"} → "*/5 * * * *"
//! - daily: {"type": "daily", "time": "09:30"} → "30 9 * * *"
//! - weekly: {"type": "weekly", "day": "monday", "time": "09:30"} → "30 9 * * 1"

/// Error type for schedule parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleError {
    InvalidJson(String),
    InvalidScheduleType(String),
    InvalidIntervalValue(u32),
    InvalidIntervalUnit(String),
    InvalidTimeFormat(String),
    InvalidDayOfWeek(String),
    MissingField(String),
}

impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScheduleError::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            ScheduleError::InvalidScheduleType(t) => write!(f, "Invalid schedule type: {}", t),
            ScheduleError::InvalidIntervalValue(v) => {
                write!(f, "Invalid interval value: {} (must be > 0)", v)
            }
            ScheduleError::InvalidIntervalUnit(u) => {
                write!(f, "Invalid interval unit: {}", u)
            }
            ScheduleError::InvalidTimeFormat(t) => {
                write!(f, "Invalid time format: {} (expected HH:MM)", t)
            }
            ScheduleError::InvalidDayOfWeek(d) => {
                write!(f, "Invalid day of week: {}", d)
            }
            ScheduleError::MissingField(field) => write!(f, "Missing required field: {}", field),
        }
    }
}

impl std::error::Error for ScheduleError {}

/// Parse a simple schedule JSON and convert to cron expression
///
/// # Examples
///
/// ```
/// use mywork_lib::scheduler::simple_schedule::parse_simple_schedule;
///
/// // Interval - every 5 minutes
/// let result = parse_simple_schedule(r#"{"type": "interval", "value": 5, "unit": "minutes"}"#);
/// assert_eq!(result.unwrap(), "*/5 * * * *");
/// ```
/// use mywork_lib::scheduler::simple_schedule::parse_simple_schedule;
///
/// // Interval - every 5 minutes
/// let result = parse_simple_schedule(r#"{"type": "interval", "value": 5, "unit": "minutes"}"#);
/// assert_eq!(result.unwrap(), "*/5 * * * *");
/// ```
pub fn parse_simple_schedule(json: &str) -> Result<String, ScheduleError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| ScheduleError::InvalidJson(e.to_string()))?;

    let schedule_type = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ScheduleError::MissingField("type".to_string()))?;

    match schedule_type {
        "interval" => parse_interval(&value),
        "daily" => parse_daily(&value),
        "weekly" => parse_weekly(&value),
        t => Err(ScheduleError::InvalidScheduleType(t.to_string())),
    }
}

/// Parse interval schedule: {"type": "interval", "value": 5, "unit": "minutes"}
fn parse_interval(value: &serde_json::Value) -> Result<String, ScheduleError> {
    let interval_value = value
        .get("value")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ScheduleError::MissingField("value".to_string()))?;

    let interval_value: u32 = interval_value
        .try_into()
        .map_err(|_| ScheduleError::InvalidIntervalValue(interval_value as u32))?;

    if interval_value == 0 {
        return Err(ScheduleError::InvalidIntervalValue(0));
    }

    let unit = value
        .get("unit")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ScheduleError::MissingField("unit".to_string()))?;

    let cron_expr = match unit {
        "minutes" => format!("*/{} * * * *", interval_value),
        "hours" => format!("0 */{} * * *", interval_value),
        "days" => format!("0 0 */{} * *", interval_value),
        u => return Err(ScheduleError::InvalidIntervalUnit(u.to_string())),
    };

    Ok(cron_expr)
}

/// Parse daily schedule: {"type": "daily", "time": "09:30"}
fn parse_daily(value: &serde_json::Value) -> Result<String, ScheduleError> {
    let time = value
        .get("time")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ScheduleError::MissingField("time".to_string()))?;

    let (hour, minute) = parse_time(time)?;

    Ok(format!("{} {} * * *", minute, hour))
}

/// Parse weekly schedule: {"type": "weekly", "day": "monday", "time": "09:30"}
fn parse_weekly(value: &serde_json::Value) -> Result<String, ScheduleError> {
    let day = value
        .get("day")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ScheduleError::MissingField("day".to_string()))?;

    let time = value
        .get("time")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ScheduleError::MissingField("time".to_string()))?;

    let (hour, minute) = parse_time(time)?;
    let day_num = parse_day_of_week(day)?;

    Ok(format!("{} {} * * {}", minute, hour, day_num))
}

/// Parse time string "HH:MM" into (hour, minute)
fn parse_time(time: &str) -> Result<(u32, u32), ScheduleError> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return Err(ScheduleError::InvalidTimeFormat(time.to_string()));
    }

    let hour: u32 = parts[0]
        .parse()
        .map_err(|_| ScheduleError::InvalidTimeFormat(time.to_string()))?;
    let minute: u32 = parts[1]
        .parse()
        .map_err(|_| ScheduleError::InvalidTimeFormat(time.to_string()))?;

    if hour > 23 {
        return Err(ScheduleError::InvalidTimeFormat(time.to_string()));
    }
    if minute > 59 {
        return Err(ScheduleError::InvalidTimeFormat(time.to_string()));
    }

    Ok((hour, minute))
}

/// Parse day of week string to cron number (0 = Sunday, 1 = Monday, etc.)
fn parse_day_of_week(day: &str) -> Result<u32, ScheduleError> {
    match day.to_lowercase().as_str() {
        "sunday" | "sun" => Ok(0),
        "monday" | "mon" => Ok(1),
        "tuesday" | "tue" => Ok(2),
        "wednesday" | "wed" => Ok(3),
        "thursday" | "thu" => Ok(4),
        "friday" | "fri" => Ok(5),
        "saturday" | "sat" => Ok(6),
        d => Err(ScheduleError::InvalidDayOfWeek(d.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ Interval Tests ============

    #[test]
    fn test_interval_minutes() {
        let json = r#"{"type": "interval", "value": 5, "unit": "minutes"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "*/5 * * * *");
    }

    #[test]
    fn test_interval_hours() {
        let json = r#"{"type": "interval", "value": 2, "unit": "hours"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "0 */2 * * *");
    }

    #[test]
    fn test_interval_days() {
        let json = r#"{"type": "interval", "value": 3, "unit": "days"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "0 0 */3 * *");
    }

    #[test]
    fn test_interval_minutes_1() {
        let json = r#"{"type": "interval", "value": 1, "unit": "minutes"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "*/1 * * * *");
    }

    #[test]
    fn test_interval_invalid_value_zero() {
        let json = r#"{"type": "interval", "value": 0, "unit": "minutes"}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ScheduleError::InvalidIntervalValue(0)
        ));
    }

    #[test]
    fn test_interval_invalid_unit() {
        let json = r#"{"type": "interval", "value": 5, "unit": "seconds"}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ScheduleError::InvalidIntervalUnit(u) if u == "seconds"
        ));
    }

    #[test]
    fn test_interval_missing_value() {
        let json = r#"{"type": "interval", "unit": "minutes"}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ScheduleError::MissingField(f) if f == "value"
        ));
    }

    #[test]
    fn test_interval_missing_unit() {
        let json = r#"{"type": "interval", "value": 5}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ScheduleError::MissingField(f) if f == "unit"
        ));
    }

    // ============ Daily Tests ============

    #[test]
    fn test_daily_morning() {
        let json = r#"{"type": "daily", "time": "09:30"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "30 9 * * *");
    }

    #[test]
    fn test_daily_midnight() {
        let json = r#"{"type": "daily", "time": "00:00"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "0 0 * * *");
    }

    #[test]
    fn test_daily_evening() {
        let json = r#"{"type": "daily", "time": "23:59"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "59 23 * * *");
    }

    #[test]
    fn test_daily_single_digit_time() {
        let json = r#"{"type": "daily", "time": "9:05"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "5 9 * * *");
    }

    #[test]
    fn test_daily_invalid_time_format() {
        let json = r#"{"type": "daily", "time": "9:30:00"}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_daily_invalid_hour() {
        let json = r#"{"type": "daily", "time": "24:00"}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_daily_invalid_minute() {
        let json = r#"{"type": "daily", "time": "12:60"}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_daily_missing_time() {
        let json = r#"{"type": "daily"}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ScheduleError::MissingField(f) if f == "time"
        ));
    }

    // ============ Weekly Tests ============

    #[test]
    fn test_weekly_monday() {
        let json = r#"{"type": "weekly", "day": "monday", "time": "09:30"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "30 9 * * 1");
    }

    #[test]
    fn test_weekly_sunday() {
        let json = r#"{"type": "weekly", "day": "sunday", "time": "14:00"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "0 14 * * 0");
    }

    #[test]
    fn test_weekly_saturday() {
        let json = r#"{"type": "weekly", "day": "saturday", "time": "10:30"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "30 10 * * 6");
    }

    #[test]
    fn test_weekly_short_day_names() {
        let json = r#"{"type": "weekly", "day": "mon", "time": "09:30"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "30 9 * * 1");
    }

    #[test]
    fn test_weekly_case_insensitive() {
        let json = r#"{"type": "weekly", "day": "MONDAY", "time": "09:30"}"#;
        let result = parse_simple_schedule(json).unwrap();
        assert_eq!(result, "30 9 * * 1");
    }

    #[test]
    fn test_weekly_invalid_day() {
        let json = r#"{"type": "weekly", "day": "funday", "time": "09:30"}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ScheduleError::InvalidDayOfWeek(d) if d == "funday"
        ));
    }

    #[test]
    fn test_weekly_missing_day() {
        let json = r#"{"type": "weekly", "time": "09:30"}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ScheduleError::MissingField(f) if f == "day"
        ));
    }

    #[test]
    fn test_weekly_missing_time() {
        let json = r#"{"type": "weekly", "day": "monday"}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ScheduleError::MissingField(f) if f == "time"
        ));
    }

    // ============ General Tests ============

    #[test]
    fn test_invalid_json() {
        let json = "not json";
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ScheduleError::InvalidJson(_)));
    }

    #[test]
    fn test_invalid_schedule_type() {
        let json = r#"{"type": "monthly", "day": 1}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ScheduleError::InvalidScheduleType(t) if t == "monthly"
        ));
    }

    #[test]
    fn test_missing_type() {
        let json = r#"{"value": 5, "unit": "minutes"}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ScheduleError::MissingField(f) if f == "type"
        ));
    }

    #[test]
    fn test_empty_object() {
        let json = r#"{}"#;
        let result = parse_simple_schedule(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_days_of_week() {
        let days = vec![
            ("sunday", 0u32),
            ("monday", 1),
            ("tuesday", 2),
            ("wednesday", 3),
            ("thursday", 4),
            ("friday", 5),
            ("saturday", 6),
        ];
        for (day, expected) in days {
            let json = format!(r#"{{"type": "weekly", "day": "{}", "time": "10:00"}}"#, day);
            let result = parse_simple_schedule(&json).unwrap();
            assert_eq!(result, format!("0 10 * * {}", expected));
        }
    }
}

use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Cron expression parsing errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CronError {
    /// Invalid number of fields (expected 5)
    InvalidFieldCount { expected: usize, actual: usize },
    /// Field value out of valid range
    OutOfRange {
        field: String,
        value: i32,
        min: i32,
        max: i32,
    },
    /// Invalid cron syntax
    InvalidSyntax { message: String },
    /// Empty expression
    EmptyExpression,
}

impl std::fmt::Display for CronError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CronError::InvalidFieldCount { expected, actual } => {
                write!(
                    f,
                    "Invalid field count: expected {}, got {}",
                    expected, actual
                )
            }
            CronError::OutOfRange {
                field,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "Field '{}' value {} out of range [{}, {}]",
                    field, value, min, max
                )
            }
            CronError::InvalidSyntax { message } => {
                write!(f, "Invalid cron syntax: {}", message)
            }
            CronError::EmptyExpression => {
                write!(f, "Cron expression cannot be empty")
            }
        }
    }
}

impl std::error::Error for CronError {}

/// Parsed cron schedule with field information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CronSchedule {
    /// Original cron expression
    pub expression: String,
    /// Minute field (0-59)
    pub minute: String,
    /// Hour field (0-23)
    pub hour: String,
    /// Day of month field (1-31)
    pub day_of_month: String,
    /// Month field (1-12)
    pub month: String,
    /// Day of week field (1-7, 1=Monday, 7=Sunday)
    pub day_of_week: String,
}

impl CronSchedule {
    /// Get a human-readable description of the schedule
    pub fn describe(&self) -> String {
        format!(
            "Minute: {}, Hour: {}, Day of Month: {}, Month: {}, Day of Week: {}",
            self.minute, self.hour, self.day_of_month, self.month, self.day_of_week
        )
    }
}

/// Validate a cron expression without returning parsed data
///
/// # Arguments
/// * `expression` - Cron expression string (5 fields)
///
/// # Returns
/// * `Ok(())` if the expression is valid
/// * `Err(CronError)` if the expression is invalid
///
/// # Example
/// ```
/// use mywork_lib::scheduler::cron_parser::validate_cron;
///
/// assert!(validate_cron("*/5 * * * *").is_ok());
/// assert!(validate_cron("invalid").is_err());
/// ```
pub fn validate_cron(expression: &str) -> Result<(), CronError> {
    let trimmed = expression.trim();

    if trimmed.is_empty() {
        return Err(CronError::EmptyExpression);
    }

    let fields: Vec<&str> = trimmed.split_whitespace().collect();

    if fields.len() != 5 {
        return Err(CronError::InvalidFieldCount {
            expected: 5,
            actual: fields.len(),
        });
    }

    let six_field_expr = format!("0 {}", trimmed);
    Schedule::from_str(&six_field_expr).map_err(|e| CronError::InvalidSyntax {
        message: format!("Failed to parse cron expression: {}", e),
    })?;

    Ok(())
}

/// Parse a cron expression and return detailed field information
///
/// # Arguments
/// * `expression` - Cron expression string (5 fields)
///
/// # Returns
/// * `Ok(CronSchedule)` with parsed field information
/// * `Err(CronError)` if the expression is invalid
///
/// # Example
/// ```
/// use mywork_lib::scheduler::cron_parser::parse_cron;
///
/// let schedule = parse_cron("0 9 * * 1-5").unwrap();
/// assert_eq!(schedule.minute, "0");
/// assert_eq!(schedule.hour, "9");
/// assert_eq!(schedule.day_of_month, "*");
/// assert_eq!(schedule.month, "*");
/// assert_eq!(schedule.day_of_week, "1-5");
/// ```
pub fn parse_cron(expression: &str) -> Result<CronSchedule, CronError> {
    let trimmed = expression.trim();

    if trimmed.is_empty() {
        return Err(CronError::EmptyExpression);
    }

    let fields: Vec<&str> = trimmed.split_whitespace().collect();

    if fields.len() != 5 {
        return Err(CronError::InvalidFieldCount {
            expected: 5,
            actual: fields.len(),
        });
    }

    let six_field_expr = format!("0 {}", trimmed);
    Schedule::from_str(&six_field_expr).map_err(|e| CronError::InvalidSyntax {
        message: format!("Failed to parse cron expression: {}", e),
    })?;

    Ok(CronSchedule {
        expression: trimmed.to_string(),
        minute: fields[0].to_string(),
        hour: fields[1].to_string(),
        day_of_month: fields[2].to_string(),
        month: fields[3].to_string(),
        day_of_week: fields[4].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_simple() {
        assert!(validate_cron("* * * * *").is_ok());
        assert!(validate_cron("*/5 * * * *").is_ok());
        assert!(validate_cron("0 * * * *").is_ok());
        assert!(validate_cron("0 0 * * *").is_ok());
        assert!(validate_cron("0 0 1 * *").is_ok());
        assert!(validate_cron("0 0 1 1 *").is_ok());
        assert!(validate_cron("0 0 * * 1").is_ok());
    }

    #[test]
    fn test_validate_valid_ranges() {
        assert!(validate_cron("0-59 * * * *").is_ok());
        assert!(validate_cron("* 0-23 * * *").is_ok());
        assert!(validate_cron("* * 1-31 * *").is_ok());
        assert!(validate_cron("* * * 1-12 *").is_ok());
        assert!(validate_cron("* * * * 1-7").is_ok());
    }

    #[test]
    fn test_validate_valid_lists() {
        assert!(validate_cron("0,15,30,45 * * * *").is_ok());
        assert!(validate_cron("* 0,6,12,18 * * *").is_ok());
        assert!(validate_cron("* * 1,15 * *").is_ok());
        assert!(validate_cron("* * * 1,6,12 *").is_ok());
        assert!(validate_cron("* * * * 1,3,5").is_ok());
    }

    #[test]
    fn test_validate_valid_steps() {
        assert!(validate_cron("*/5 * * * *").is_ok());
        assert!(validate_cron("* */2 * * *").is_ok());
        assert!(validate_cron("* * */5 * *").is_ok());
        assert!(validate_cron("* * * */3 *").is_ok());
        assert!(validate_cron("* * * * */2").is_ok());
    }

    #[test]
    fn test_validate_valid_complex_common_patterns() {
        assert!(validate_cron("0 9 * * 1-5").is_ok());
        assert!(validate_cron("*/15 9-17 * * 1-5").is_ok());
        assert!(validate_cron("0 0 1 1 *").is_ok());
        assert!(validate_cron("0 0 29 2 *").is_ok());
        assert!(validate_cron("0,30 0,12 * * *").is_ok());
    }

    #[test]
    fn test_validate_with_whitespace() {
        assert!(validate_cron("  * * * * *  ").is_ok());
        assert!(validate_cron("*  *  *  *  *").is_ok());
        assert!(validate_cron("\t*\t*\t*\t*\t*").is_ok());
    }

    #[test]
    fn test_validate_empty() {
        assert_eq!(validate_cron(""), Err(CronError::EmptyExpression));
        assert_eq!(validate_cron("   "), Err(CronError::EmptyExpression));
        assert_eq!(validate_cron("\t\n"), Err(CronError::EmptyExpression));
    }

    #[test]
    fn test_validate_invalid_field_count() {
        assert!(matches!(
            validate_cron("*"),
            Err(CronError::InvalidFieldCount {
                expected: 5,
                actual: 1
            })
        ));
        assert!(matches!(
            validate_cron("* *"),
            Err(CronError::InvalidFieldCount {
                expected: 5,
                actual: 2
            })
        ));
        assert!(matches!(
            validate_cron("* * *"),
            Err(CronError::InvalidFieldCount {
                expected: 5,
                actual: 3
            })
        ));
        assert!(matches!(
            validate_cron("* * * *"),
            Err(CronError::InvalidFieldCount {
                expected: 5,
                actual: 4
            })
        ));
        assert!(matches!(
            validate_cron("* * * * * *"),
            Err(CronError::InvalidFieldCount {
                expected: 5,
                actual: 6
            })
        ));
        assert!(matches!(
            validate_cron("* * * * * * *"),
            Err(CronError::InvalidFieldCount {
                expected: 5,
                actual: 7
            })
        ));
    }

    #[test]
    fn test_validate_invalid_syntax() {
        assert!(matches!(
            validate_cron("a * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("* abc * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("60 * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("* 24 * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("* * 0 * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("* * 32 * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("* * * 0 *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("* * * 13 *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("* * * * 0"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("* * * * 8"),
            Err(CronError::InvalidSyntax { .. })
        ));
    }

    #[test]
    fn test_validate_invalid_range_syntax() {
        assert!(matches!(
            validate_cron("30-0 * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("* 23-0 * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("- * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("0- * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("-0 * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
    }

    #[test]
    fn test_validate_invalid_step_syntax() {
        assert!(matches!(
            validate_cron("/* * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("*/ * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("*/0 * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
    }

    #[test]
    fn test_validate_invalid_list_syntax() {
        assert!(matches!(
            validate_cron(", * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("0, * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron(",0 * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
        assert!(matches!(
            validate_cron("0,,1 * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
    }

    #[test]
    fn test_parse_simple() {
        let schedule = parse_cron("* * * * *").unwrap();
        assert_eq!(schedule.expression, "* * * * *");
        assert_eq!(schedule.minute, "*");
        assert_eq!(schedule.hour, "*");
        assert_eq!(schedule.day_of_month, "*");
        assert_eq!(schedule.month, "*");
        assert_eq!(schedule.day_of_week, "*");
    }

    #[test]
    fn test_parse_specific_values() {
        let schedule = parse_cron("30 14 15 6 3").unwrap();
        assert_eq!(schedule.minute, "30");
        assert_eq!(schedule.hour, "14");
        assert_eq!(schedule.day_of_month, "15");
        assert_eq!(schedule.month, "6");
        assert_eq!(schedule.day_of_week, "3");
    }

    #[test]
    fn test_parse_range() {
        let schedule = parse_cron("0-30 9-17 1-15 1-6 1-5").unwrap();
        assert_eq!(schedule.minute, "0-30");
        assert_eq!(schedule.hour, "9-17");
        assert_eq!(schedule.day_of_month, "1-15");
        assert_eq!(schedule.month, "1-6");
        assert_eq!(schedule.day_of_week, "1-5");
    }

    #[test]
    fn test_parse_list() {
        let schedule = parse_cron("0,30 9,17 1,15 1,6 1,3").unwrap();
        assert_eq!(schedule.minute, "0,30");
        assert_eq!(schedule.hour, "9,17");
        assert_eq!(schedule.day_of_month, "1,15");
        assert_eq!(schedule.month, "1,6");
        assert_eq!(schedule.day_of_week, "1,3");
    }

    #[test]
    fn test_parse_step() {
        let schedule = parse_cron("*/5 */2 */3 */4 */1").unwrap();
        assert_eq!(schedule.minute, "*/5");
        assert_eq!(schedule.hour, "*/2");
        assert_eq!(schedule.day_of_month, "*/3");
        assert_eq!(schedule.month, "*/4");
        assert_eq!(schedule.day_of_week, "*/1");
    }

    #[test]
    fn test_parse_complex() {
        let schedule = parse_cron("*/15 9-17 * * 1-5").unwrap();
        assert_eq!(schedule.minute, "*/15");
        assert_eq!(schedule.hour, "9-17");
        assert_eq!(schedule.day_of_month, "*");
        assert_eq!(schedule.month, "*");
        assert_eq!(schedule.day_of_week, "1-5");
    }

    #[test]
    fn test_parse_preserves_expression() {
        let expr = "  0  9  *  *  1-5  ";
        let schedule = parse_cron(expr).unwrap();
        assert_eq!(schedule.expression, "0  9  *  *  1-5");
    }

    #[test]
    fn test_parse_empty() {
        assert_eq!(parse_cron(""), Err(CronError::EmptyExpression));
    }

    #[test]
    fn test_parse_invalid_field_count() {
        assert!(matches!(
            parse_cron("* * *"),
            Err(CronError::InvalidFieldCount {
                expected: 5,
                actual: 3
            })
        ));
    }

    #[test]
    fn test_parse_invalid_syntax() {
        assert!(matches!(
            parse_cron("60 * * * *"),
            Err(CronError::InvalidSyntax { .. })
        ));
    }

    #[test]
    fn test_parse_boundary_values() {
        assert!(parse_cron("0 0 1 1 1").is_ok());
        assert!(parse_cron("59 23 31 12 7").is_ok());
        assert!(parse_cron("* * * * *").is_ok());
    }

    #[test]
    fn test_parse_common_patterns() {
        assert!(parse_cron("* * * * *").is_ok());
        assert!(parse_cron("0 * * * *").is_ok());
        assert!(parse_cron("0 0 * * *").is_ok());
        assert!(parse_cron("0 0 * * 7").is_ok());
        assert!(parse_cron("0 0 1 * *").is_ok());
        assert!(parse_cron("*/5 * * * *").is_ok());
        assert!(parse_cron("*/15 * * * *").is_ok());
        assert!(parse_cron("*/30 * * * *").is_ok());
        assert!(parse_cron("0 0,12 * * *").is_ok());
        assert!(parse_cron("0 9 * * 1-5").is_ok());
    }

    #[test]
    fn test_describe() {
        let schedule = parse_cron("*/15 9-17 * * 1-5").unwrap();
        let description = schedule.describe();
        assert!(description.contains("*/15"));
        assert!(description.contains("9-17"));
        assert!(description.contains("1-5"));
    }

    #[test]
    fn test_cron_error_display() {
        let err = CronError::InvalidFieldCount {
            expected: 5,
            actual: 3,
        };
        assert_eq!(err.to_string(), "Invalid field count: expected 5, got 3");

        let err = CronError::OutOfRange {
            field: "minute".to_string(),
            value: 60,
            min: 0,
            max: 59,
        };
        assert_eq!(
            err.to_string(),
            "Field 'minute' value 60 out of range [0, 59]"
        );

        let err = CronError::InvalidSyntax {
            message: "test error".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid cron syntax: test error");

        let err = CronError::EmptyExpression;
        assert_eq!(err.to_string(), "Cron expression cannot be empty");
    }

    #[test]
    fn test_cron_error_is_std_error() {
        let err = CronError::EmptyExpression;
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_cron_schedule_serialization() {
        let schedule = parse_cron("0 9 * * 1-5").unwrap();
        let json = serde_json::to_string(&schedule).unwrap();
        let deserialized: CronSchedule = serde_json::from_str(&json).unwrap();
        assert_eq!(schedule, deserialized);
    }

    #[test]
    fn test_cron_error_serialization() {
        let err = CronError::InvalidFieldCount {
            expected: 5,
            actual: 3,
        };
        let json = serde_json::to_string(&err).unwrap();
        let deserialized: CronError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, deserialized);
    }
}

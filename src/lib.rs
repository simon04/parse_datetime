// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use regex::{Error as RegexError, Regex};
use std::error::Error;
use std::fmt::{self, Display};
use time::Duration;

#[derive(Debug)]
pub enum ParseDurationError {
    InvalidRegex(RegexError),
    InvalidInput,
}

impl Display for ParseDurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseDurationError::InvalidRegex(err) => {
                write!(f, "Invalid regex for time pattern: {err}")
            }
            ParseDurationError::InvalidInput => {
                write!(
                    f,
                    "Invalid input string: cannot be parsed as a relative time"
                )
            }
        }
    }
}

impl Error for ParseDurationError {}

impl From<RegexError> for ParseDurationError {
    fn from(err: RegexError) -> Self {
        ParseDurationError::InvalidRegex(err)
    }
}

/// Parses a relative time string and returns a `Duration` representing the
/// relative time.
///
/// # Arguments
///
/// * `s` - A string slice representing the relative time.
///
/// # Examples
///
/// ```
/// use time::Duration;
/// let duration = humantime_to_duration::from_str("+3 days");
/// assert_eq!(duration.unwrap(), Duration::days(3));
/// ```
///
/// # Supported formats
///
/// The function supports the following formats for relative time:
///
/// * `num` `unit` (e.g., "-1 hour", "+3 days")
/// * `unit` (e.g., "hour", "day")
/// * "now" or "today"
/// * "yesterday"
/// * "tomorrow"
/// * use "ago" for the past
///
/// [num] can be a positive or negative integer.
/// [unit] can be one of the following: "fortnight", "week", "day", "hour",
/// "minute", "min", "second", "sec" and their plural forms.
///
/// It is also possible to pass "1 hour 2 minutes" or "2 days and 2 hours"
///
/// # Returns
///
/// * `Some(Duration)` - If the input string can be parsed as a relative time
/// * `None` - If the input string cannot be parsed as a relative time
///
/// # Errors
///
/// This function will return `None` if the input string cannot be parsed as a
/// relative time.
///
/// # Panics
///
/// This function will return `None` if the input string cannot be parsed as a
/// relative time.
pub fn from_str(s: &str) -> Result<Duration, ParseDurationError> {
    let time_pattern: Regex = Regex::new(
            r"(?x)
            (?P<value>[-+]?\d*)\s*
            (?P<unit>years?|months?|fortnights?|weeks?|days?|hours?|h|minutes?|mins?|m|seconds?|secs?|s|yesterday|tomorrow|now|today)
            (\s*(?P<separator>and|,)?\s*)?"
        )
        .unwrap();

    let mut total_duration = Duration::ZERO;
    let mut is_ago = s.contains(" ago");

    for capture in time_pattern.captures_iter(s) {
        let value_str = capture.name("value").unwrap().as_str();
        let value = if value_str.is_empty() {
            1
        } else {
            value_str.parse::<i64>().unwrap_or(1)
        };
        let unit = capture.name("unit").unwrap().as_str();

        if capture.name("ago").is_some() {
            is_ago = true;
        }

        let duration = match unit {
            "years" | "year" => Duration::days(value * 365),
            "months" | "month" => Duration::days(value * 30),
            "fortnights" | "fortnight" => Duration::weeks(value * 2),
            "weeks" | "week" => Duration::weeks(value),
            "days" | "day" => Duration::days(value),
            "hours" | "hour" | "h" => Duration::hours(value),
            "minutes" | "minute" | "mins" | "min" | "m" => Duration::minutes(value),
            "seconds" | "second" | "secs" | "sec" | "s" => Duration::seconds(value),
            "yesterday" => Duration::days(-1),
            "tomorrow" => Duration::days(1),
            "now" | "today" => Duration::ZERO,
            _ => return Err(ParseDurationError::InvalidInput),
        };

        total_duration = match total_duration.checked_add(if is_ago { -duration } else { duration })
        {
            Some(duration) => duration,
            None => return Err(ParseDurationError::InvalidInput),
        };
    }

    if total_duration == Duration::ZERO && !time_pattern.is_match(s) {
        Err(ParseDurationError::InvalidInput)
    } else {
        Ok(total_duration)
    }
}

#[cfg(test)]
mod tests {

    use super::from_str;
    use super::ParseDurationError;
    use time::Duration;

    #[test]
    fn test_years() {
        assert_eq!(from_str("1 year").unwrap(), Duration::seconds(31_536_000));
        assert_eq!(
            from_str("-2 years").unwrap(),
            Duration::seconds(-63_072_000)
        );
        assert_eq!(
            from_str("2 years ago").unwrap(),
            Duration::seconds(-63_072_000)
        );
        assert_eq!(from_str("year").unwrap(), Duration::seconds(31_536_000));
    }

    #[test]
    fn test_months() {
        assert_eq!(from_str("1 month").unwrap(), Duration::seconds(2_592_000));
        assert_eq!(
            from_str("1 month and 2 weeks").unwrap(),
            Duration::seconds(3_801_600)
        );
        assert_eq!(
            from_str("1 month and 2 weeks ago").unwrap(),
            Duration::seconds(-3_801_600)
        );
        assert_eq!(from_str("2 months").unwrap(), Duration::seconds(5_184_000));
        assert_eq!(from_str("month").unwrap(), Duration::seconds(2_592_000));
    }

    #[test]
    fn test_fortnights() {
        assert_eq!(
            from_str("1 fortnight").unwrap(),
            Duration::seconds(1_209_600)
        );
        assert_eq!(
            from_str("3 fortnights").unwrap(),
            Duration::seconds(3_628_800)
        );
        assert_eq!(from_str("fortnight").unwrap(), Duration::seconds(1_209_600));
    }

    #[test]
    fn test_weeks() {
        assert_eq!(from_str("1 week").unwrap(), Duration::seconds(604_800));
        assert_eq!(
            from_str("1 week 3 days").unwrap(),
            Duration::seconds(864_000)
        );
        assert_eq!(
            from_str("1 week 3 days ago").unwrap(),
            Duration::seconds(-864_000)
        );
        assert_eq!(from_str("-2 weeks").unwrap(), Duration::seconds(-1_209_600));
        assert_eq!(
            from_str("2 weeks ago").unwrap(),
            Duration::seconds(-1_209_600)
        );
        assert_eq!(from_str("week").unwrap(), Duration::seconds(604_800));
    }

    #[test]
    fn test_days() {
        assert_eq!(from_str("1 day").unwrap(), Duration::seconds(86400));
        assert_eq!(from_str("2 days ago").unwrap(), Duration::seconds(-172_800));
        assert_eq!(from_str("-2 days").unwrap(), Duration::seconds(-172_800));
        assert_eq!(from_str("day").unwrap(), Duration::seconds(86400));
    }

    #[test]
    fn test_hours() {
        assert_eq!(from_str("1 hour").unwrap(), Duration::seconds(3600));
        assert_eq!(from_str("1 hour ago").unwrap(), Duration::seconds(-3600));
        assert_eq!(from_str("-2 hours").unwrap(), Duration::seconds(-7200));
        assert_eq!(from_str("hour").unwrap(), Duration::seconds(3600));
    }

    #[test]
    fn test_minutes() {
        assert_eq!(from_str("1 minute").unwrap(), Duration::seconds(60));
        assert_eq!(from_str("2 minutes").unwrap(), Duration::seconds(120));
        assert_eq!(from_str("min").unwrap(), Duration::seconds(60));
    }

    #[test]
    fn test_seconds() {
        assert_eq!(from_str("1 second").unwrap(), Duration::seconds(1));
        assert_eq!(from_str("2 seconds").unwrap(), Duration::seconds(2));
        assert_eq!(from_str("sec").unwrap(), Duration::seconds(1));
    }

    #[test]
    fn test_relative_days() {
        assert_eq!(from_str("now").unwrap(), Duration::seconds(0));
        assert_eq!(from_str("today").unwrap(), Duration::seconds(0));
        assert_eq!(from_str("yesterday").unwrap(), Duration::seconds(-86400));
        assert_eq!(from_str("tomorrow").unwrap(), Duration::seconds(86400));
    }

    #[test]
    fn test_no_spaces() {
        assert_eq!(from_str("-1hour").unwrap(), Duration::hours(-1));
        assert_eq!(from_str("+3days").unwrap(), Duration::days(3));
        assert_eq!(from_str("2weeks").unwrap(), Duration::weeks(2));
        assert_eq!(
            from_str("2weeks 1hour").unwrap(),
            Duration::seconds(1_213_200)
        );
        assert_eq!(
            from_str("2weeks 1hour ago").unwrap(),
            Duration::seconds(-1_213_200)
        );
        assert_eq!(from_str("+4months").unwrap(), Duration::days(4 * 30));
        assert_eq!(from_str("-2years").unwrap(), Duration::days(-2 * 365));
        assert_eq!(from_str("15minutes").unwrap(), Duration::minutes(15));
        assert_eq!(from_str("-30seconds").unwrap(), Duration::seconds(-30));
        assert_eq!(from_str("30seconds ago").unwrap(), Duration::seconds(-30));
    }

    #[test]
    fn test_invalid_input() {
        let result = from_str("foobar");
        println!("{result:?}");
        match result {
            Err(ParseDurationError::InvalidInput) => assert!(true),
            _ => assert!(false),
        }

        let result = from_str("invalid 1");
        match result {
            Err(ParseDurationError::InvalidInput) => assert!(true),
            _ => assert!(false),
        }
    }
}

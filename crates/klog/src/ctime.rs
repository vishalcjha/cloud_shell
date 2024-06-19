use std::str::FromStr;

use chrono::{DateTime, TimeDelta, Utc};
use clap::builder::TypedValueParser;
use regex::Regex;
pub type UtcTime = DateTime<Utc>;
#[derive(Debug, Clone)]
pub(crate) enum CTime {
    Before(UtcTime),
    After(UtcTime),
}

impl CTime {
    pub fn is_valid(&self, other: &UtcTime) -> bool {
        match self {
            CTime::Before(time) => other.cmp(time).is_le(),
            CTime::After(time) => other.cmp(time).is_ge(),
        }
    }
}

impl AsRef<UtcTime> for CTime {
    fn as_ref(&self) -> &UtcTime {
        match self {
            CTime::Before(x) => x,
            CTime::After(x) => x,
        }
    }
}

impl FromStr for CTime {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^([\+]?)([0-9]+)([h,m,s])$").or(Err("Incorrect regex for time"))?;

        let captured = re
            .captures(s)
            .ok_or("Incorrect format : Use '+10s|m|h' '10s|m|h'")?;

        let time = captured.get(2).unwrap().as_str().parse::<i64>().unwrap();
        let measurement = captured.get(3).unwrap().as_str();
        let current_time = Utc::now();
        let time: CTime = if captured.get(1).unwrap().as_str() == "" {
            match measurement {
                "m" => Self::Before(
                    current_time
                        .checked_sub_signed(TimeDelta::minutes(time))
                        .unwrap(),
                ),
                "h" => Self::Before(
                    current_time
                        .checked_sub_signed(TimeDelta::hours(time))
                        .unwrap(),
                ),
                "s" => Self::Before(
                    current_time
                        .checked_sub_signed(TimeDelta::seconds(time))
                        .unwrap(),
                ),
                _ => panic!(),
            }
        } else {
            match measurement {
                "m" => Self::After(
                    current_time
                        .checked_add_signed(TimeDelta::minutes(time))
                        .unwrap(),
                ),
                "h" => Self::After(
                    current_time
                        .checked_add_signed(TimeDelta::hours(time))
                        .unwrap(),
                ),
                "s" => Self::After(
                    current_time
                        .checked_add_signed(TimeDelta::seconds(time))
                        .unwrap(),
                ),
                _ => panic!(),
            }
        };

        Ok(time)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CTimeParser;

impl TypedValueParser for CTimeParser {
    type Value = CTime;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value = value.to_str().unwrap();

        CTime::from_str(value).map_err(|err| {
            let mut clap_error =
                clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(cmd);
            clap_error.insert(
                clap::error::ContextKind::InvalidValue,
                clap::error::ContextValue::String(err.to_string()),
            );
            clap_error
        })
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use chrono::Utc;

    use super::CTime;

    #[test]
    fn test_valid_before() {
        let ctime = CTime::from_str("12h");
        assert!(ctime.is_ok());
        assert!(Utc::now().cmp(ctime.unwrap().as_ref()).is_gt())
    }

    #[test]
    fn test_valid_after() {
        let ctime = CTime::from_str("+12h");
        assert!(ctime.is_ok());
        assert!(Utc::now().cmp(ctime.unwrap().as_ref()).is_lt())
    }

    #[test]
    fn test_invalid_char_at_beginning() {
        let ctime = CTime::from_str("-13h");
        assert!(ctime.is_err());
    }

    #[test]
    fn test_invalid_measurement() {
        let ctime = CTime::from_str("12x");
        assert!(ctime.is_err());
    }

    #[test]
    fn test_invalid_numeric() {
        let ctime = CTime::from_str("1h2h");
        assert!(ctime.is_err());
    }
}

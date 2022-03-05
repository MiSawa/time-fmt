use std::fmt::Write;

use thiserror::Error;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};
#[cfg(feature = "timezone_name")]
use time_tz::{Offset, PrimitiveDateTimeExt, TimeZone, Tz};

use crate::{format::spec_parser::Collector, util};

mod spec_parser;
pub mod time_format_item;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Unknown specifier `%{0}`")]
    UnknownSpecifier(char),
    #[error(transparent)]
    Format(#[from] std::fmt::Error),
}

struct FormatCollector<'a, W: Write> {
    date: Date,
    time: Time,
    offset: Option<UtcOffset>,
    #[cfg(feature = "timezone_name")]
    zone: Option<<Tz as TimeZone>::Offset>,
    write: &'a mut W,
}
impl<'a, W: Write> FormatCollector<'a, W> {
    fn from_date_time(date_time: PrimitiveDateTime, write: &'a mut W) -> Self {
        Self {
            date: date_time.date(),
            time: date_time.time(),
            offset: None,
            #[cfg(feature= "timezone_name")]
            zone: None,
            write,
        }
    }
    fn from_offset_date_time(date_time: OffsetDateTime, write: &'a mut W) -> Self {
        Self {
            date: date_time.date(),
            time: date_time.time(),
            offset: Some(date_time.offset()),
            #[cfg(feature= "timezone_name")]
            zone: None,
            write,
        }
    }

    #[cfg(feature = "timezone_name")]
    fn from_zoned_date_time(date_time: PrimitiveDateTime, zone: &'a Tz, write: &'a mut W) -> Self {
        let offset_datetime = date_time.assume_timezone(zone);
        let offset: <Tz as TimeZone>::Offset = zone.get_offset_utc(&offset_datetime);
        Self {
            date: date_time.date(),
            time: date_time.time(),
            offset: Some(offset.to_utc()),
            zone: Some(offset),
            write,
        }
    }
}

impl<'a, W: Write> Collector for FormatCollector<'a, W> {
    type Output = ();
    type Error = Error;

    #[inline]
    fn day_of_week_name_short(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_str(util::weekday_short_str(self.date.weekday()))?;
        Ok(())
    }

    #[inline]
    fn day_of_week_name_long(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_str(util::weekday_long_str(self.date.weekday()))?;
        Ok(())
    }

    #[inline]
    fn month_name_short(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_str(util::month_short_str(self.date.month()))?;
        Ok(())
    }

    #[inline]
    fn month_name_long(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_str(util::month_long_str(self.date.month()))?;
        Ok(())
    }

    #[inline]
    fn year_prefix(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:02}", self.date.year().div_euclid(100)))?;
        Ok(())
    }

    #[inline]
    fn day_of_month(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:02}", self.date.day()))?;
        Ok(())
    }

    #[inline]
    fn day_of_month_blank(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:2}", self.date.day()))?;
        Ok(())
    }

    #[inline]
    fn iso8601_week_based_year_suffix(&mut self) -> Result<(), Self::Error> {
        let (year, _, _) = self.date.to_iso_week_date();
        if year >= 0 {
            self.write.write_fmt(format_args!("{:02}", year / 100))?;
        } else {
            self.write.write_fmt(format_args!("-{}", (-year) / 100))?;
        }
        Ok(())
    }

    #[inline]
    fn iso8601_week_based_year(&mut self) -> Result<(), Self::Error> {
        let (year, _, _) = self.date.to_iso_week_date();
        self.write.write_fmt(format_args!("{:4}", year))?;
        Ok(())
    }

    #[inline]
    fn hour_of_day(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:02}", self.time.hour()))?;
        Ok(())
    }

    #[inline]
    fn hour_of_day_12(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:02}", (self.time.hour() + 11) % 12 + 1))?;
        Ok(())
    }

    #[inline]
    fn day_of_year(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:03}", self.date.ordinal()))?;
        Ok(())
    }

    #[inline]
    fn hour_of_day_blank(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:2}", self.time.hour()))?;
        Ok(())
    }

    #[inline]
    fn hour_of_day_12_blank(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:2}", (self.time.hour() + 11) % 12 + 1))?;
        Ok(())
    }

    #[inline]
    fn month_of_year(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:02}", self.date.month() as u8))?;
        Ok(())
    }

    #[inline]
    fn minute_of_hour(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:02}", self.time.minute()))?;
        Ok(())
    }

    #[inline]
    fn ampm(&mut self) -> Result<(), Self::Error> {
        self.write.write_str(util::ampm_upper(self.time.hour()))?;
        Ok(())
    }

    #[inline]
    fn ampm_lower(&mut self) -> Result<(), Self::Error> {
        self.write.write_str(util::ampm_lower(self.time.hour()))?;
        Ok(())
    }

    #[inline]
    fn second_of_minute(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:02}", self.time.second()))?;
        Ok(())
    }

    #[inline]
    fn day_of_week_from_monday_as_1(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{}", self.date.weekday().number_from_monday()))?;
        Ok(())
    }

    #[inline]
    fn week_number_of_current_year_start_sunday(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:02}", self.date.sunday_based_week()))?;
        Ok(())
    }

    #[inline]
    fn iso8601_week_number(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:02}", self.date.iso_week()))?;
        Ok(())
    }

    #[inline]
    fn day_of_week_from_sunday_as_0(&mut self) -> Result<(), Self::Error> {
        self.write.write_fmt(format_args!(
            "{}",
            self.date.weekday().number_days_from_sunday()
        ))?;
        Ok(())
    }

    #[inline]
    fn week_number_of_current_year_start_monday(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:02}", self.date.monday_based_week()))?;
        Ok(())
    }

    #[inline]
    fn year_suffix(&mut self) -> Result<(), Self::Error> {
        let year = self.date.year();
        self.write
            .write_fmt(format_args!("{:02}", year.abs() % 100))?;
        Ok(())
    }

    #[inline]
    fn year(&mut self) -> Result<(), Self::Error> {
        self.write
            .write_fmt(format_args!("{:04}", self.date.year()))?;
        Ok(())
    }

    #[inline]
    fn timezone(&mut self) -> Result<(), Self::Error> {
        if let Some(offset) = self.offset {
            let (h, m, _) = offset.as_hms();
            if offset.is_negative() {
                self.write.write_fmt(format_args!("-{:02}{:02}", -h, -m))?;
            } else {
                self.write.write_fmt(format_args!("+{:02}{:02}", h, m))?;
            }
        }
        // No bytes if no timezone is determinable.
        Ok(())
    }

    #[inline]
    fn timezone_name(&mut self) -> Result<(), Self::Error> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "timezone_name")] {
                if let Some(zone) = &self.zone {
                    self.write.write_str(zone.name())?;
                }
            }
        }
        // No bytes if no timezone information exists.
        Ok(())
    }

    #[inline]
    fn static_str(&mut self, s: &'static str) -> Result<(), Self::Error> {
        self.write.write_str(s)?;
        Ok(())
    }

    #[inline]
    fn literal(
        &mut self,
        lit: &str,
        _fmt_span: impl std::slice::SliceIndex<[u8], Output = [u8]>,
    ) -> Result<(), Self::Error> {
        self.write.write_str(lit)?;
        Ok(())
    }

    #[inline]
    fn unknown(&mut self, specifier: char) -> Result<(), Self::Error> {
        Err(Self::Error::UnknownSpecifier(specifier))
    }

    #[inline]
    fn output(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub fn format_primitive_date_time(
    fmt: &str,
    date_time: PrimitiveDateTime,
) -> Result<String, Error> {
    let mut ret = String::new();
    let collector = FormatCollector::from_date_time(date_time, &mut ret);
    spec_parser::parse_conversion_specifications(fmt, collector)?;
    Ok(ret)
}

pub fn format_offset_date_time(fmt: &str, date_time: OffsetDateTime) -> Result<String, Error> {
    let mut ret = String::new();
    let collector = FormatCollector::from_offset_date_time(date_time, &mut ret);
    spec_parser::parse_conversion_specifications(fmt, collector)?;
    Ok(ret)
}

#[cfg(feature= "timezone_name")]
pub fn format_zoned_date_time(
    fmt: &str,
    date_time: PrimitiveDateTime,
    zone: &'static Tz,
) -> Result<String, Error> {
    let mut ret = String::new();
    let collector = FormatCollector::from_zoned_date_time(date_time, zone, &mut ret);
    spec_parser::parse_conversion_specifications(fmt, collector)?;
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::format_offset_date_time;
    use time::macros::datetime;

    #[test]
    fn test_year_prefix() -> Result<(), super::Error> {
        let fmt = "%C";
        assert_eq!(
            format_offset_date_time(fmt, datetime!(410-01-01 01:01:01 UTC))?,
            "04".to_string()
        );
        assert_eq!(
            format_offset_date_time(fmt, datetime!(2021-01-01 01:01:01 UTC))?,
            "20".to_string()
        );
        assert_eq!(
            format_offset_date_time(fmt, datetime!(+99999-01-01 01:01:01 UTC))?,
            "999".to_string()
        );
        assert_eq!(
            format_offset_date_time(fmt, datetime!(-1-01-01 01:01:01 UTC))?,
            "-1".to_string()
        );
        assert_eq!(
            format_offset_date_time(fmt, datetime!(-1000-01-01 01:01:01 UTC))?,
            "-10".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_offset() -> Result<(), super::Error> {
        let fmt = "%z";
        assert_eq!(
            format_offset_date_time(fmt, datetime!(410-01-01 01:01:01 UTC))?,
            "+0000".to_string()
        );
        assert_eq!(
            format_offset_date_time(fmt, datetime!(2022-02-02 01:01:01 -1:23))?,
            "-0123".to_string()
        );
        Ok(())
    }

    #[cfg(feature = "timezone_name")]
    #[test]
    fn test_timezone_name() -> Result<(), super::Error> {
        use super::format_zoned_date_time;
        use time_tz::timezones;
        let tokyo = timezones::db::asia::TOKYO;

        assert_eq!(
            format_zoned_date_time("%z %Z", datetime!(2022-02-02 02:02:02), tokyo)?,
            "+0900 JST".to_string()
        );
        Ok(())
    }
}

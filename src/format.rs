use std::fmt::Write;

use thiserror::Error;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

use crate::{format::spec_parser::Collector, util};

mod spec_parser;
pub mod time_format_item;

#[derive(Error, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FormatError {
    #[error("Unknown specifier `%{0}`")]
    UnknownSpecifier(char),
    #[error(transparent)]
    Format(#[from] std::fmt::Error),
}

struct FormatCollector<'a, W: Write> {
    date: Date,
    time: Time,
    offset: Option<UtcOffset>,
    zone_name: Option<&'a str>,
    write: &'a mut W,
}
impl<'a, W: Write> FormatCollector<'a, W> {
    fn from_date_time(date_time: PrimitiveDateTime, write: &'a mut W) -> Self {
        Self {
            date: date_time.date(),
            time: date_time.time(),
            offset: None,
            zone_name: None,
            write,
        }
    }
    fn from_offset_date_time(date_time: OffsetDateTime, write: &'a mut W) -> Self {
        Self {
            date: date_time.date(),
            time: date_time.time(),
            offset: Some(date_time.offset()),
            zone_name: None,
            write,
        }
    }

    fn from_zoned_date_time(
        date_time: PrimitiveDateTime,
        offset: UtcOffset,
        zone_name: &'a str,
        write: &'a mut W,
    ) -> Self {
        Self {
            date: date_time.date(),
            time: date_time.time(),
            offset: Some(offset),
            zone_name: Some(zone_name),
            write,
        }
    }

    fn from_zoned_offset_date_time(
        date_time: OffsetDateTime,
        zone_name: &'a str,
        write: &'a mut W,
    ) -> Self {
        Self {
            date: date_time.date(),
            time: date_time.time(),
            offset: Some(date_time.offset()),
            zone_name: Some(zone_name),
            write,
        }
    }
}

impl<'a, W: Write> Collector for FormatCollector<'a, W> {
    type Output = ();
    type Error = FormatError;

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
        self.write
            .write_fmt(format_args!("{:02}", year.rem_euclid(100)))?;
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
    fn nanosecond_of_minute(&mut self) -> Result<(), Self::Error> {
        let nanoseconds = self.time.nanosecond();

        let keep_digits: usize = if nanoseconds % 10 != 0 {
            9
        } else if (nanoseconds / 10) % 10 != 0 {
            8
        } else if (nanoseconds / 100) % 10 != 0 {
            7
        } else if (nanoseconds / 1_000) % 10 != 0 {
            6
        } else if (nanoseconds / 10_000) % 10 != 0 {
            5
        } else if (nanoseconds / 100_000) % 10 != 0 {
            4
        } else if (nanoseconds / 1_000_000) % 10 != 0 {
            3
        } else if (nanoseconds / 10_000_000) % 10 != 0 {
            2
        } else if (nanoseconds / 100_000_000) % 10 != 0 {
            1
        } else {
            0
        };

        let nanos_string = nanoseconds.to_string();
        let zeros_padding: usize = if nanos_string.len() == 9 { 0 } else { 9 };

        self.write
            .write_fmt(
                format_args!("{:0>padding$.precision$}", 
                nanos_string, precision = keep_digits, padding = zeros_padding)
            )?;

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
        if let Some(zone_name) = &self.zone_name {
            self.write.write_str(zone_name)?;
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

pub fn format_date_time(fmt: &str, date_time: PrimitiveDateTime) -> Result<String, FormatError> {
    let mut ret = String::new();
    let collector = FormatCollector::from_date_time(date_time, &mut ret);
    spec_parser::parse_conversion_specifications(fmt, collector)?;
    Ok(ret)
}

pub fn format_offset_date_time(
    fmt: &str,
    date_time: OffsetDateTime,
) -> Result<String, FormatError> {
    let mut ret = String::new();
    let collector = FormatCollector::from_offset_date_time(date_time, &mut ret);
    spec_parser::parse_conversion_specifications(fmt, collector)?;
    Ok(ret)
}

pub fn format_zoned_date_time(
    fmt: &str,
    date_time: PrimitiveDateTime,
    offset: UtcOffset,
    zone_name: &str,
) -> Result<String, FormatError> {
    let mut ret = String::new();
    let collector = FormatCollector::from_zoned_date_time(date_time, offset, zone_name, &mut ret);
    spec_parser::parse_conversion_specifications(fmt, collector)?;
    Ok(ret)
}

pub fn format_zoned_offset_date_time(
    fmt: &str,
    date_time: OffsetDateTime,
    zone_name: &str,
) -> Result<String, FormatError> {
    let mut ret = String::new();
    let collector = FormatCollector::from_zoned_offset_date_time(date_time, zone_name, &mut ret);
    spec_parser::parse_conversion_specifications(fmt, collector)?;
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::{format_date_time, format_offset_date_time};
    use time::{
        macros::{datetime, offset},
        PrimitiveDateTime,
    };

    #[test]
    fn test_simple() -> Result<(), super::FormatError> {
        fn test_datetime(
            fmt: &str,
            dt: PrimitiveDateTime,
            expected: &str,
        ) -> Result<(), super::FormatError> {
            assert_eq!(format_date_time(fmt, dt)?, expected);
            assert_eq!(
                format_offset_date_time(fmt, dt.assume_offset(offset!(+9:00)))?,
                expected
            );
            assert_eq!(
                super::format_zoned_date_time(fmt, dt, offset!(+9:00), "JST")?,
                expected
            );
            assert_eq!(
                super::format_zoned_offset_date_time(fmt, dt.assume_offset(offset!(+9:00)), "JST")?,
                expected
            );
            Ok(())
        }

        let datetime = datetime!(2022-03-06 12:34:56);
        let datetime2 = datetime!(2022-03-06 02:04:06);
        test_datetime("%a %A", datetime, "Sun Sunday")?;
        test_datetime("%b %h %B", datetime, "Mar Mar March")?;
        test_datetime("%c", datetime, "Sun Mar  6 12:34:56 2022")?;
        test_datetime("%C", datetime, "20")?;
        test_datetime("%d", datetime, "06")?;
        test_datetime("%D", datetime, "03/06/22")?;
        test_datetime("%e", datetime, " 6")?;
        test_datetime("%F", datetime, "2022-03-06")?;
        test_datetime("%g", datetime, "22")?;
        test_datetime("%G", datetime, "2022")?;
        test_datetime("%H", datetime, "12")?;
        test_datetime("%H", datetime2, "02")?;
        test_datetime("%I", datetime, "12")?;
        test_datetime("%I", datetime2, "02")?;
        test_datetime("%j", datetime, "065")?;
        test_datetime("%k", datetime2, " 2")?;
        test_datetime("%l", datetime, "12")?;
        test_datetime("%l", datetime2, " 2")?;
        test_datetime("%m", datetime, "03")?;
        test_datetime("%M", datetime, "34")?;
        test_datetime("%n", datetime, "\n")?;
        test_datetime("%p", datetime, "PM")?;
        test_datetime("%P", datetime, "pm")?;
        test_datetime("%r", datetime, "12:34:56 PM")?;
        test_datetime("%r", datetime2, "02:04:06 AM")?;
        test_datetime("%R", datetime, "12:34")?;
        test_datetime("%R", datetime2, "02:04")?;
        test_datetime("%S", datetime, "56")?;
        test_datetime("%t", datetime, "\t")?;
        test_datetime("%T", datetime, "12:34:56")?;
        test_datetime("%u", datetime, "7")?;
        test_datetime("%U", datetime, "10")?;
        test_datetime("%V", datetime, "09")?;
        test_datetime("%w", datetime, "0")?;
        test_datetime("%W", datetime, "09")?;
        test_datetime("%x", datetime, "03/06/22")?;
        test_datetime("%X", datetime, "12:34:56")?;
        test_datetime("%y", datetime, "22")?;
        test_datetime("%Y", datetime, "2022")?;
        test_datetime("%%", datetime, "%")?;

        let datetime_ms1 = datetime!(2022-03-06 02:04:06.1);
        let datetime_ms2 = datetime!(2022-03-06 02:04:06.12);
        let datetime_ms3 = datetime!(2022-03-06 02:04:06.123);
        let datetime_ms4 = datetime!(2022-03-06 02:04:06.1234);
        let datetime_ms5 = datetime!(2022-03-06 02:04:06.12345);
        let datetime_ms6 = datetime!(2022-03-06 02:04:06.123456);
        let datetime_ms7 = datetime!(2022-03-06 02:04:06.1234567);
        let datetime_ms8 = datetime!(2022-03-06 02:04:06.12345678);
        let datetime_ms9 = datetime!(2022-03-06 02:04:06.123456789);

        test_datetime("%f", datetime_ms1, "1")?;
        test_datetime("%f", datetime_ms2, "12")?;
        test_datetime("%f", datetime_ms3, "123")?;
        test_datetime("%f", datetime_ms4, "1234")?;
        test_datetime("%f", datetime_ms5, "12345")?;
        test_datetime("%f", datetime_ms6, "123456")?;
        test_datetime("%f", datetime_ms7, "1234567")?;
        test_datetime("%f", datetime_ms8, "12345678")?;
        test_datetime("%f", datetime_ms9, "123456789")?;

        let datetime_ms1 = datetime!(2022-03-06 02:04:06.900000000);
        let datetime_ms2 = datetime!(2022-03-06 02:04:06.980000000);
        let datetime_ms3 = datetime!(2022-03-06 02:04:06.987000000);
        let datetime_ms4 = datetime!(2022-03-06 02:04:06.987600000);
        let datetime_ms5 = datetime!(2022-03-06 02:04:06.987650000);
        let datetime_ms6 = datetime!(2022-03-06 02:04:06.987654000);
        let datetime_ms7 = datetime!(2022-03-06 02:04:06.987654300);
        let datetime_ms8 = datetime!(2022-03-06 02:04:06.987654320);

        test_datetime("%f", datetime_ms1, "9")?;
        test_datetime("%f", datetime_ms2, "98")?;
        test_datetime("%f", datetime_ms3, "987")?;
        test_datetime("%f", datetime_ms4, "9876")?;
        test_datetime("%f", datetime_ms5, "98765")?;
        test_datetime("%f", datetime_ms6, "987654")?;
        test_datetime("%f", datetime_ms7, "9876543")?;
        test_datetime("%f", datetime_ms8, "98765432")?;

        let datetime_ms1 = datetime!(2022-03-06 02:04:06.000000002);
        let datetime_ms2 = datetime!(2022-03-06 02:04:06.000000022);
        let datetime_ms3 = datetime!(2022-03-06 02:04:06.000000222);
        let datetime_ms4 = datetime!(2022-03-06 02:04:06.000002222);
        let datetime_ms5 = datetime!(2022-03-06 02:04:06.000022222);
        let datetime_ms6 = datetime!(2022-03-06 02:04:06.000222222);
        let datetime_ms7 = datetime!(2022-03-06 02:04:06.002222222);
        let datetime_ms8 = datetime!(2022-03-06 02:04:06.022222222);

        test_datetime("%f", datetime_ms1, "000000002")?;
        test_datetime("%f", datetime_ms2, "000000022")?;
        test_datetime("%f", datetime_ms3, "000000222")?;
        test_datetime("%f", datetime_ms4, "000002222")?;
        test_datetime("%f", datetime_ms5, "000022222")?;
        test_datetime("%f", datetime_ms6, "000222222")?;
        test_datetime("%f", datetime_ms7, "002222222")?;
        test_datetime("%f", datetime_ms8, "022222222")?;

        Ok(())
    }

    #[test]
    fn test_year_prefix() -> Result<(), super::FormatError> {
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
    fn test_offset() -> Result<(), super::FormatError> {
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

    #[test]
    fn test_timezone_name() -> Result<(), super::FormatError> {
        use super::{format_zoned_date_time, format_zoned_offset_date_time};

        assert_eq!(
            format_zoned_date_time(
                "%z %Z",
                datetime!(2022-02-02 02:02:02),
                offset!(+9:00),
                "JST"
            )?,
            "+0900 JST".to_string()
        );

        assert_eq!(
            format_zoned_offset_date_time(
                "%T %z %Z",
                datetime!(2022-02-02 02:02:02 UTC).to_offset(offset!(+9:00)),
                "JST"
            )?,
            "11:02:02 +0900 JST".to_string()
        );
        Ok(())
    }
}

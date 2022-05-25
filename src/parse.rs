use thiserror::Error;
use time::{Date, Month, PrimitiveDateTime, Time, UtcOffset, Weekday};

use crate::{parse::desc_parser::Collector, util};

mod desc_parser;

#[derive(Error, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    #[error("Unknown specifier `%{0}`")]
    UnknownSpecifier(char),
    #[error("Expected {0} but got a byte {1}")]
    UnexpectedByte(&'static str, u8),
    #[error("Expected {0} but reached to the end")]
    UnexpectedEnd(&'static str),
    #[error("Expected {0} but doesn't have a match")]
    NotMatch(&'static str),
    #[error("Out-of-range for {0} component")]
    ComponentOutOfRange(&'static str),
    #[error("Unconverted data remains: {0}")]
    UnconvertedDataRemains(String),
    #[error(transparent)]
    ComponentRange(#[from] time::error::ComponentRange),
}

trait Nat: std::ops::Add<Output = Self> + std::ops::Mul<Output = Self>
where
    Self: Sized,
{
    const ZERO: Self;
    const TEN: Self;
    fn from_u8(_: u8) -> Self;
}

trait Int: Nat + std::ops::Neg<Output = Self> {}

impl Nat for u8 {
    const ZERO: Self = 0;
    const TEN: Self = 10;
    fn from_u8(v: u8) -> Self {
        v
    }
}
impl Nat for u16 {
    const ZERO: Self = 0;
    const TEN: Self = 10;
    fn from_u8(v: u8) -> Self {
        v as u16
    }
}
impl Nat for u32 {
    const ZERO: Self = 0;
    const TEN: Self = 10;
    fn from_u8(v: u8) -> Self {
        v as u32
    }
}
impl Nat for i16 {
    const ZERO: Self = 0;
    const TEN: Self = 10;
    fn from_u8(v: u8) -> Self {
        v as i16
    }
}
impl Int for i16 {}
impl Nat for i32 {
    const ZERO: Self = 0;
    const TEN: Self = 10;
    fn from_u8(v: u8) -> Self {
        v as i32
    }
}
impl Int for i32 {}

#[derive(Debug)]
enum ParsingYear {
    Unspecified,
    Year(i32),
    PrefixSuffix(i32, u8),
}
#[derive(Debug)]
enum ParsingDayOfYear {
    Unspecified,
    MonthDay(Month, u8),
    DayOfYear(u16),
}
#[derive(Debug)]
enum ParsingHour {
    Unspecified,
    FullDay(u8),
    HalfDay(u8, bool),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TimeZoneSpecifier<'a> {
    Offset(UtcOffset),
    Name(&'a str),
}

struct ParseCollector<'a> {
    s: &'a str,
    year: ParsingYear,
    day: ParsingDayOfYear,
    hour: ParsingHour,
    minute: u8,
    second: u8,
    nanosecond: u32,
    zone: Option<TimeZoneSpecifier<'a>>,
}
impl<'a> ParseCollector<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            s,
            year: ParsingYear::Unspecified,
            day: ParsingDayOfYear::Unspecified,
            hour: ParsingHour::Unspecified,
            minute: 0,
            second: 0,
            nanosecond: 0,
            zone: None,
        }
    }

    #[inline]
    fn skip_whitespaces(&mut self) {
        self.s = self.s.trim_start();
    }

    #[inline]
    fn get_until_whitespace(&mut self) -> &'a str {
        let pos = self.s.find(char::is_whitespace).unwrap_or(self.s.len());
        let (res, rest) = self.s.split_at(pos);
        self.s = rest;
        res
    }

    #[inline]
    fn peek_byte(&self) -> Option<u8> {
        self.s.bytes().next()
    }

    /// Note: Need a change if pass max_len that makes us require checking for overflow.
    #[inline]
    fn parse_nat<N: Nat>(&mut self, min_len: usize, max_len: usize) -> Result<N, ParseError> {
        if self.s.len() < min_len {
            return Err(ParseError::UnexpectedEnd("digits"));
        }
        let bytes = self.s.as_bytes();
        let max_len = max_len.min(bytes.len());
        let mut res = N::ZERO;
        let mut bytes_read = 0;
        for &c in &bytes[..max_len] {
            if (b'0'..=b'9').contains(&c) {
                res = (res * N::TEN) + N::from_u8(c - b'0');
            } else if bytes_read < min_len {
                return Err(ParseError::UnexpectedByte("digits", c));
            } else {
                break;
            }
            bytes_read += 1;
        }
        self.s = &self.s[bytes_read..];
        Ok(res)
    }

    /// Allows '+'/'-'.
    /// Note: Need a change if pass max_len that makes us require checking for overflow.
    #[inline]
    fn parse_int<Z: Int>(&mut self, max_len: usize) -> Result<Z, ParseError> {
        if self.s.is_empty() {
            return Err(ParseError::UnexpectedEnd("digits"));
        }
        let max_len = max_len.min(self.s.len());
        let mut res = Z::ZERO;
        let mut bytes_read = 0;
        let mut negate = false;
        let mut had_digit = false;
        for &c in &self.s.as_bytes()[..max_len] {
            if (b'0'..=b'9').contains(&c) {
                res = (res * Z::TEN) + Z::from_u8(c - b'0');
                had_digit = true;
            } else if bytes_read == 0 {
                if c == b'+' {
                    // skip it
                } else if c == b'-' {
                    negate = true;
                } else {
                    return Err(ParseError::UnexpectedByte("digits or sign", c));
                }
            } else if had_digit {
                break;
            } else {
                return Err(ParseError::UnexpectedByte("digits", c));
            }
            bytes_read += 1;
        }
        self.s = &self.s[bytes_read..];
        Ok(if negate { -res } else { res })
    }

    #[inline]
    fn starts_with_ignore_ascii_case(&self, prefix: &str) -> bool {
        self.s.len() >= prefix.len() && self.s.is_char_boundary(prefix.len()) && self.s[..prefix.len()].eq_ignore_ascii_case(prefix)
    }
}

impl<'a> Collector for ParseCollector<'a> {
    type Output = (PrimitiveDateTime, Option<TimeZoneSpecifier<'a>>);
    type Error = ParseError;

    #[inline]
    fn spaces(&mut self) -> Result<(), Self::Error> {
        self.skip_whitespaces();
        Ok(())
    }

    #[inline]
    fn day_of_week_name(&mut self) -> Result<(), Self::Error> {
        let mut weekday = Weekday::Monday;
        for _i in 0..7 {
            let short = util::weekday_short_str(weekday);
            if self.starts_with_ignore_ascii_case(short) {
                let long = util::weekday_long_str(weekday);
                if self.starts_with_ignore_ascii_case(long) {
                    self.s = &self.s[long.len()..];
                } else {
                    self.s = &self.s[short.len()..];
                }
                // Found match. Ignore it!
                return Ok(());
            }
            weekday = weekday.next();
        }
        Err(Self::Error::NotMatch("day of week name"))
    }

    #[inline]
    fn month_name(&mut self) -> Result<(), Self::Error> {
        let mut month = Month::January;
        for _i in 0..12 {
            let short = util::month_short_str(month);
            if self.starts_with_ignore_ascii_case(short) {
                let long = util::month_long_str(month);
                if self.starts_with_ignore_ascii_case(long) {
                    self.s = &self.s[long.len()..];
                } else {
                    self.s = &self.s[short.len()..];
                }
                match &mut self.day {
                    ParsingDayOfYear::Unspecified => {
                        self.day = ParsingDayOfYear::MonthDay(month, 1)
                    }
                    ParsingDayOfYear::MonthDay(current, _) => *current = month,
                    // Prefer day of year over (month, day)
                    ParsingDayOfYear::DayOfYear(_) => {}
                }
                return Ok(());
            }
            month = month.next();
        }
        Err(Self::Error::NotMatch("month name"))
    }

    #[inline]
    fn year_prefix(&mut self) -> Result<(), Self::Error> {
        let prefix = self.parse_int(2)?;
        match &mut self.year {
            ParsingYear::Unspecified => self.year = ParsingYear::PrefixSuffix(prefix, 0),
            // Prefer year over (year prefix, year suffix).
            ParsingYear::Year(_) => {}
            ParsingYear::PrefixSuffix(v, _) => *v = prefix,
        }
        Ok(())
    }

    #[inline]
    fn day_of_month(&mut self) -> Result<(), Self::Error> {
        let day = self.parse_nat(1, 2)?;
        if (1..=31).contains(&day) {
            match &mut self.day {
                ParsingDayOfYear::Unspecified => {
                    self.day = ParsingDayOfYear::MonthDay(Month::January, day)
                }
                ParsingDayOfYear::MonthDay(_, current) => *current = day,
                // Prefer day of year over (month, day)
                ParsingDayOfYear::DayOfYear(_) => {}
            }
            Ok(())
        } else {
            Err(Self::Error::ComponentOutOfRange("day-of-month"))
        }
    }

    #[inline]
    fn hour_of_day(&mut self) -> Result<(), Self::Error> {
        let hour = self.parse_nat(1, 2)?;
        if (0..24).contains(&hour) {
            match &mut self.hour {
                ParsingHour::Unspecified => self.hour = ParsingHour::FullDay(hour),
                ParsingHour::FullDay(current) => *current = hour,
                // Prefer full day over halfday + am/pm.
                ParsingHour::HalfDay(_, _) => {}
            }
            Ok(())
        } else {
            Err(Self::Error::ComponentOutOfRange("hour-of-day"))
        }
    }

    #[inline]
    fn hour_of_day_12(&mut self) -> Result<(), Self::Error> {
        let hour: u8 = self.parse_nat(1, 2)?;
        if (1..=12).contains(&hour) {
            let hour = hour % 12;
            match &mut self.hour {
                ParsingHour::Unspecified => self.hour = ParsingHour::HalfDay(hour, false),
                // Prefer full day over halfday + am/pm.
                ParsingHour::FullDay(_) => {}
                ParsingHour::HalfDay(current, _) => *current = hour,
            }
            Ok(())
        } else {
            Err(Self::Error::ComponentOutOfRange("hour-of-half-day"))
        }
    }

    #[inline]
    fn day_of_year(&mut self) -> Result<(), Self::Error> {
        let day = self.parse_nat(1, 3)?;
        if (1..=366).contains(&day) {
            // Prefer day of year over (month, day)
            self.day = ParsingDayOfYear::DayOfYear(day);
            Ok(())
        } else {
            Err(Self::Error::ComponentOutOfRange("day-of-year"))
        }
    }

    #[inline]
    fn month_of_year(&mut self) -> Result<(), Self::Error> {
        let month = self.parse_nat(1, 2)?;
        if (1..=12).contains(&month) {
            let month = util::get_month(month).unwrap();
            match &mut self.day {
                ParsingDayOfYear::Unspecified => self.day = ParsingDayOfYear::MonthDay(month, 1),
                ParsingDayOfYear::MonthDay(current, _) => *current = month,
                // Prefer day of year over (month, day)
                ParsingDayOfYear::DayOfYear(_) => {}
            }
            Ok(())
        } else {
            Err(Self::Error::ComponentOutOfRange("month"))
        }
    }

    #[inline]
    fn minute_of_hour(&mut self) -> Result<(), Self::Error> {
        let minute = self.parse_nat(1, 2)?;
        if (0..60).contains(&minute) {
            self.minute = minute;
            Ok(())
        } else {
            Err(Self::Error::ComponentOutOfRange("munute"))
        }
    }

    #[inline]
    fn ampm(&mut self) -> Result<(), Self::Error> {
        for h in [0, 12] {
            let s = util::ampm_lower(h);
            if self.starts_with_ignore_ascii_case(s) {
                match &mut self.hour {
                    ParsingHour::Unspecified => self.hour = ParsingHour::HalfDay(0, h != 0),
                    // Prefer full day over halfday + am/pm.
                    ParsingHour::FullDay(_) => {}
                    ParsingHour::HalfDay(_, current) => *current = h != 0,
                }
                // Consume AM/PM substring
                self.s = &self.s[2..];
                return Ok(());
            }
        }
        Err(Self::Error::NotMatch("am/pm"))
    }

    #[inline]
    fn second_of_minute(&mut self) -> Result<(), Self::Error> {
        let second = self.parse_nat(1, 2)?;
        if (0..61).contains(&second) {
            self.second = second;
            Ok(())
        } else {
            Err(Self::Error::ComponentOutOfRange("second"))
        }
    }

    #[inline]
    fn nanosecond_of_second(&mut self) -> Result<(), Self::Error> {
        let input_length = self.s.len();
        let nanosecond: u32 = self.parse_nat(1, 9)?;

        let digits_consumed = input_length - self.s.len();
        static SCALE: [u32; 10] = [
            0, 100_000_000, 10_000_000, 1_000_000, 100_000, 10_000, 1_000, 100, 10, 1
        ];
        self.nanosecond = nanosecond * SCALE[digits_consumed];

        Ok(())
    }

    #[inline]
    fn week_number_of_current_year_start_sunday(&mut self) -> Result<(), Self::Error> {
        let w: u8 = self.parse_nat(1, 2)?;
        if (0..=53).contains(&w) {
            // Ignore it!
            Ok(())
        } else {
            Err(Self::Error::ComponentOutOfRange("week-number"))
        }
    }

    #[inline]
    fn day_of_week_from_sunday_as_0(&mut self) -> Result<(), Self::Error> {
        let w: u8 = self.parse_nat(1, 1)?;
        if (0..7).contains(&w) {
            // Ignore it!
            Ok(())
        } else {
            Err(Self::Error::ComponentOutOfRange("day-of-week"))
        }
    }

    #[inline]
    fn week_number_of_current_year_start_monday(&mut self) -> Result<(), Self::Error> {
        let w: u8 = self.parse_nat(1, 2)?;
        if (0..=53).contains(&w) {
            Ok(())
        } else {
            Err(Self::Error::ComponentOutOfRange("week-number"))
        }
    }

    #[inline]
    fn year_suffix(&mut self) -> Result<(), Self::Error> {
        let y = self.parse_nat(1, 2)?;
        if (0..100).contains(&y) {
            match &mut self.year {
                ParsingYear::Unspecified => {
                    self.year = ParsingYear::PrefixSuffix(if y < 69 { 20 } else { 19 }, y)
                }
                // Prefer year over (year prefix, year suffix).
                ParsingYear::Year(_) => {}
                ParsingYear::PrefixSuffix(_, current) => *current = y,
            }
            Ok(())
        } else {
            Err(Self::Error::ComponentOutOfRange("year-suffix"))
        }
    }

    #[inline]
    fn year(&mut self) -> Result<(), Self::Error> {
        let y = self.parse_int(4)?;
        // Prefer year over (year prefix, year suffix).
        self.year = ParsingYear::Year(y);
        Ok(())
    }

    #[inline]
    fn timezone(&mut self) -> Result<(), Self::Error> {
        let negate = match self.peek_byte() {
            Some(b'Z') => {
                self.s = &self.s[1..]; // skip Z
                self.zone = Some(TimeZoneSpecifier::Offset(UtcOffset::UTC));
                return Ok(());
            }
            Some(c @ (b'+' | b'-')) => {
                self.s = &self.s[1..]; // skip sign
                c == b'-'
            }
            Some(b) => return Err(Self::Error::UnexpectedByte("+ or -", b)),
            None => return Err(Self::Error::UnexpectedEnd("+ or -")),
        };
        let h: u8 = self.parse_nat(2, 2)?;
        if self.peek_byte() == Some(b':') {
            self.s = &self.s[1..]; // skip :
        }
        let m: u8 = self.parse_nat(2, 2)?;
        let h: i8 = h
            .try_into()
            .map_err(|_| Self::Error::ComponentOutOfRange("offset-hour"))?;
        let m: i8 = m
            .try_into()
            .map_err(|_| Self::Error::ComponentOutOfRange("offset-minute"))?;
        let (h, m) = if negate { (-h, -m) } else { (h, m) };
        self.zone = Some(TimeZoneSpecifier::Offset(UtcOffset::from_hms(h, m, 0)?));
        Ok(())
    }

    #[inline]
    fn timezone_name(&mut self) -> Result<(), Self::Error> {
        let s = self.get_until_whitespace();
        self.zone = Some(TimeZoneSpecifier::Name(s));
        Ok(())
    }

    #[inline]
    fn static_str(&mut self, s: &'static str) -> Result<(), Self::Error> {
        if let Some(rest) = self.s.strip_prefix(s) {
            self.s = rest;
            Ok(())
        } else {
            Err(Self::Error::NotMatch(s))
        }
    }

    #[inline]
    fn literal(
        &mut self,
        lit: &str,
        _fmt_span: impl std::slice::SliceIndex<[u8], Output = [u8]>,
    ) -> Result<(), Self::Error> {
        if let Some(rest) = self.s.strip_prefix(lit) {
            self.s = rest;
            Ok(())
        } else {
            Err(Self::Error::NotMatch("string literal"))
        }
    }

    #[inline]
    fn unknown(&mut self, specifier: char) -> Result<(), Self::Error> {
        Err(Self::Error::UnknownSpecifier(specifier))
    }

    #[inline]
    fn unconsumed_input(&self) -> Result<(), Self::Error> {
        let unconsumed_input = self.s.to_string();
        if unconsumed_input.len() > 0 {
            Err(Self::Error::UnconvertedDataRemains(unconsumed_input))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn output(self) -> Result<Self::Output, Self::Error> {
        let year = match self.year {
            ParsingYear::Unspecified => 1900,
            ParsingYear::Year(y) => y,
            ParsingYear::PrefixSuffix(p, s) => p
                .checked_mul(100)
                .and_then(|p| p.checked_add(s as i32))
                .ok_or(Self::Error::ComponentOutOfRange("year"))?,
        };
        let date = match self.day {
            ParsingDayOfYear::Unspecified => Date::from_ordinal_date(year, 1)?,
            ParsingDayOfYear::MonthDay(month, day) => Date::from_calendar_date(year, month, day)?,
            ParsingDayOfYear::DayOfYear(day) => Date::from_ordinal_date(year, day)?,
        };
        let hour = match self.hour {
            ParsingHour::Unspecified => 0,
            ParsingHour::FullDay(h) => h,
            ParsingHour::HalfDay(h, ampm) => {
                if ampm {
                    h + 12
                } else {
                    h
                }
            }
        };
        let time = Time::from_hms_nano(hour, self.minute, self.second, self.nanosecond)?;
        let zone = self.zone;
        Ok((PrimitiveDateTime::new(date, time), zone))
    }
}

pub fn parse_date_time_maybe_with_zone<'a>(
    fmt: &str,
    s: &'a str,
) -> Result<(PrimitiveDateTime, Option<TimeZoneSpecifier<'a>>), ParseError> {
    let collector = ParseCollector::new(s);
    desc_parser::parse_format_specifications(fmt, collector, false)
}

pub fn parse_strict_date_time_maybe_with_zone<'a>(
    fmt: &str,
    s: &'a str,
) -> Result<(PrimitiveDateTime, Option<TimeZoneSpecifier<'a>>), ParseError> {
    let collector = ParseCollector::new(s);
    desc_parser::parse_format_specifications(fmt, collector, true)
}

#[cfg(test)]
mod tests {
    use super::{parse_date_time_maybe_with_zone, parse_strict_date_time_maybe_with_zone, ParseError, TimeZoneSpecifier};
    use time::macros::{datetime, offset};

    #[test]
    fn test_simple_parse() -> Result<(), super::ParseError> {
        assert_eq!(
            parse_date_time_maybe_with_zone("%a %A %a", "wED Wed weDnesDay")?,
            (datetime!(1900-01-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%b %B %b", "feB FEb feburaRy")?,
            (datetime!(1900-02-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%c", "Sun Mar  6 12:34:56 2022")?,
            (datetime!(2022-03-06 12:34:56), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%C", "20")?,
            (datetime!(2000-01-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%d", "5")?,
            (datetime!(1900-01-05 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%e", "5")?,
            (datetime!(1900-01-05 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%D", "3 /6/22")?,
            (datetime!(2022-03-06 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%F", "2022-03-06")?,
            (datetime!(2022-03-06 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%H", "2")?,
            (datetime!(1900-01-01 2:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%k", "2")?,
            (datetime!(1900-01-01 2:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%I", "2")?,
            (datetime!(1900-01-01 2:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%l", "12")?,
            (datetime!(1900-01-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%j", "38")?,
            (datetime!(1900-02-07 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%m", "8")?,
            (datetime!(1900-08-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%M", "8")?,
            (datetime!(1900-01-01 00:08:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%n%t ", "   ")?,
            (datetime!(1900-01-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%I %p", "12 AM")?,
            (datetime!(1900-01-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%I %p", "1 AM")?,
            (datetime!(1900-01-01 01:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%I %p", "1 pm")?,
            (datetime!(1900-01-01 13:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%I %p", "12 pm")?,
            (datetime!(1900-01-01 12:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%r", "12:34:56 PM")?,
            (datetime!(1900-01-01 12:34:56), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%r", "12:34:56 AM")?,
            (datetime!(1900-01-01 00:34:56), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%r %F", "12:34:56 AM 2022-03-06")?,
            (datetime!(2022-03-06 00:34:56), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%R", "12: 4")?,
            (datetime!(1900-01-01 12:04:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%S", "01")?,
            (datetime!(1900-01-01 00:00:01), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%T", "01:23:45")?,
            (datetime!(1900-01-01 01:23:45), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%T.%f", "01:23:45.123")?,
            (datetime!(1900-01-01 01:23:45.123), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%T.%f", "01:23:45.12300")?,
            (datetime!(1900-01-01 01:23:45.123), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%T.%f", "01:23:45.000001234")?,
            (datetime!(1900-01-01 01:23:45.000001234), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%U", "1")?,
            (datetime!(1900-01-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%w", "1")?,
            (datetime!(1900-01-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%W", "1")?,
            (datetime!(1900-01-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%x", "3/6/22")?,
            (datetime!(2022-03-06 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%X", "12:34:5")?,
            (datetime!(1900-01-01 12:34:05), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%y", "12")?,
            (datetime!(2012-01-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%y", "70")?,
            (datetime!(1970-01-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%Y", "70")?,
            (datetime!(0070-01-01 00:00:00), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%C%y", "2022")?,
            (datetime!(2022-01-01 00:00:00), None)
        );
        Ok(())
    }

    #[test]
    fn test_strict() {
        assert!(
            matches!(
                parse_strict_date_time_maybe_with_zone("%F", "2022-03-06T12:34:56Z"),
                Err(ParseError::UnconvertedDataRemains(_)),
            )
        );

        assert_eq!(
            parse_strict_date_time_maybe_with_zone("%FT%TZ", "2022-03-06T12:34:56Z"),
            Ok((datetime!(2022-03-06 12:34:56), None))
        );
    }

    #[test]
    fn test_zone() -> Result<(), super::ParseError> {
        assert_eq!(
            parse_date_time_maybe_with_zone("%FT%TZ", "2022-03-06T12:34:56Z")?,
            (datetime!(2022-03-06 12:34:56), None)
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%FT%T%z", "2022-03-06T12:34:56Z")?,
            (
                datetime!(2022-03-06 12:34:56),
                Some(TimeZoneSpecifier::Offset(offset!(+00:00)))
            )
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%FT%T%Z", "2022-03-06T12:34:56Z")?,
            (
                datetime!(2022-03-06 12:34:56),
                Some(TimeZoneSpecifier::Name("Z"))
            )
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%FT%T %z", "2022-03-06T12:34:56 -1234")?,
            (
                datetime!(2022-03-06 12:34:56),
                Some(TimeZoneSpecifier::Offset(offset!(-12:34)))
            )
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%FT%T %Z", "2022-03-06T12:34:56 JST")?,
            (
                datetime!(2022-03-06 12:34:56),
                Some(TimeZoneSpecifier::Name("JST"))
            )
        );
        assert_eq!(
            parse_date_time_maybe_with_zone("%FT%T %z", "2022-03-06T12:34:56 -12:34")?,
            (
                datetime!(2022-03-06 12:34:56),
                Some(TimeZoneSpecifier::Offset(offset!(-12:34)))
            )
        );
        assert!(parse_date_time_maybe_with_zone("%FT%T %z", "2022-03-06T12:34:56 12:34").is_err());
        assert!(parse_date_time_maybe_with_zone("%FT%T %z", "2022-03-06T12:34:56 +2:34").is_err());
        assert!(parse_date_time_maybe_with_zone("%FT%T %z", "2022-03-06T12:34:56 +234").is_err());
        Ok(())
    }
}

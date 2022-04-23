use std::slice::SliceIndex;

/// E and O are not implemented.
/// Those require `nl-langinfo` lookup is default-implemented as if it were a POSIX locale.
/// If you'd want to implement it properly, it's your responsibility to recursively parse
/// the format you get from `nl-langinfo`, and prevent infinite recursion.
pub(crate) trait Collector {
    type Output;
    type Error;
    /// Skips sequence of whitespaces.
    fn spaces(&mut self) -> Result<(), Self::Error>;
    /// `%a` or `%A`. `nl_langinfo`-dependent.
    fn day_of_week_name(&mut self) -> Result<(), Self::Error>;
    /// `%b`, `%B` or `%h`. `nl_langinfo`-dependent.
    fn month_name(&mut self) -> Result<(), Self::Error>;
    /// `%c`. Same as `%a %b %e %T %Y` in POSIX locale. `nl_langinfo`-dependent.
    #[inline]
    fn preferred_date_time(&mut self) -> Result<(), Self::Error> {
        self.day_of_week_name()?;
        self.spaces()?;
        self.month_name()?;
        self.spaces()?;
        self.day_of_month()?;
        self.spaces()?;
        self.time_of_day()?;
        self.spaces()?;
        self.year()
    }
    /// `%C`. `0` to `99`.
    fn year_prefix(&mut self) -> Result<(), Self::Error>;
    /// `%d`, `%e`. `01` to `31`.
    fn day_of_month(&mut self) -> Result<(), Self::Error>;
    /// `%D`. `%m / %d / %y` (American......).
    #[inline]
    fn date_mmddyy_slash(&mut self) -> Result<(), Self::Error> {
        self.month_of_year()?;
        self.spaces()?;
        self.static_str("/")?;
        self.spaces()?;
        self.day_of_month()?;
        self.spaces()?;
        self.static_str("/")?;
        self.spaces()?;
        self.year_suffix()
    }
    /// `%F`. `%Y-%m-%d`.
    #[inline]
    fn date_yyyymmdd_hyphen(&mut self) -> Result<(), Self::Error> {
        self.year()?;
        self.static_str("-")?;
        self.month_of_year()?;
        self.static_str("-")?;
        self.day_of_month()
    }
    /// `%H`, `%k`. `00` to `23`.
    fn hour_of_day(&mut self) -> Result<(), Self::Error>;
    /// `%I`, `%l`. `01` to `12`.
    fn hour_of_day_12(&mut self) -> Result<(), Self::Error>;
    /// `%j`. `001` to `336`.
    fn day_of_year(&mut self) -> Result<(), Self::Error>;
    /// `%m`. `01` to `12`.
    fn month_of_year(&mut self) -> Result<(), Self::Error>;
    /// `%M`. `00` to `59`.
    fn minute_of_hour(&mut self) -> Result<(), Self::Error>;
    /// `%n`.
    #[inline]
    fn new_line(&mut self) -> Result<(), Self::Error> {
        self.spaces()
    }
    /// `%p`, `%P`. `AM` or `PM`. `nl_langinfo`-dependent.
    fn ampm(&mut self) -> Result<(), Self::Error>;
    /// `%r`. Same as `%I : %M : %S %p` in POSIX locale. `nl_langinfo`-dependent.
    #[inline]
    fn time_ampm(&mut self) -> Result<(), Self::Error> {
        self.hour_of_day_12()?;
        self.spaces()?;
        self.static_str(":")?;
        self.spaces()?;
        self.minute_of_hour()?;
        self.spaces()?;
        self.static_str(":")?;
        self.spaces()?;
        self.second_of_minute()?;
        self.spaces()?;
        self.ampm()
    }
    /// `%R`. Same as `%H : %M`.
    #[inline]
    fn hour_minute_of_day(&mut self) -> Result<(), Self::Error> {
        self.hour_of_day()?;
        self.spaces()?;
        self.static_str(":")?;
        self.spaces()?;
        self.minute_of_hour()
    }
    /// `%S`. `00` to `60`.
    fn second_of_minute(&mut self) -> Result<(), Self::Error>;
    /// `%f`. `000000000` to `999999999`.
    fn nanosecond_of_minute(&mut self) -> Result<(), Self::Error>;
    /// `%t`.
    #[inline]
    fn tab(&mut self) -> Result<(), Self::Error> {
        self.spaces()
    }
    /// `%T`. Same as `%H : %M : %S`.
    #[inline]
    fn time_of_day(&mut self) -> Result<(), Self::Error> {
        self.hour_of_day()?;
        self.spaces()?;
        self.static_str(":")?;
        self.spaces()?;
        self.minute_of_hour()?;
        self.spaces()?;
        self.static_str(":")?;
        self.spaces()?;
        self.second_of_minute()
    }
    /// `%U`. `00` to `53`.
    fn week_number_of_current_year_start_sunday(&mut self) -> Result<(), Self::Error>;
    /// `%w`.
    fn day_of_week_from_sunday_as_0(&mut self) -> Result<(), Self::Error>;
    /// `%W`. `00` to `53`.
    fn week_number_of_current_year_start_monday(&mut self) -> Result<(), Self::Error>;
    /// `%x`. `%m/%d/%y` in POSIX locale. `nl_langinfo`-dependent.
    #[inline]
    fn preferred_date(&mut self) -> Result<(), Self::Error> {
        self.month_of_year()?;
        self.static_str("/")?;
        self.day_of_month()?;
        self.static_str("/")?;
        self.year_suffix()
    }
    /// `%X`. `%H:%M:%S` in POSIX locale. `nl_langinfo`-dependent.
    #[inline]
    fn preferred_time_of_day(&mut self) -> Result<(), Self::Error> {
        self.hour_of_day()?;
        self.static_str(":")?;
        self.minute_of_hour()?;
        self.static_str(":")?;
        self.second_of_minute()
    }
    /// `%y`. `00` to `99`.
    fn year_suffix(&mut self) -> Result<(), Self::Error>;
    /// `%Y`.
    fn year(&mut self) -> Result<(), Self::Error>;
    /// `%z`. `+hhmm` or `-hhmm`.
    fn timezone(&mut self) -> Result<(), Self::Error>;
    /// `%Z`. Timezone name or abbreviation.
    fn timezone_name(&mut self) -> Result<(), Self::Error>;
    /// `%%`.
    #[inline]
    fn percent(&mut self) -> Result<(), Self::Error> {
        self.static_str("%")
    }
    /// Escaped character or seprators in formatted string like `:` or `/`.
    /// It's just a character but we'd want a &'static str.
    fn static_str(&mut self, s: &'static str) -> Result<(), Self::Error>;
    /// Other literals.
    /// The byte range of the original format `fmt_span` is passed so that you can internally store
    /// the original format and index that to get the same content with `lit` with your favorite
    /// lifetime.
    fn literal(
        &mut self,
        lit: &str,
        fmt_span: impl SliceIndex<[u8], Output = [u8]>,
    ) -> Result<(), Self::Error>;
    /// `%(something else)`.
    fn unknown(&mut self, specifier: char) -> Result<(), Self::Error>;

    /// Construct the final result from what you've collected.
    fn output(self) -> Result<Self::Output, Self::Error>;
}

pub(crate) fn parse_format_specifications<C: Collector>(
    mut format: &str,
    mut collector: C,
) -> Result<C::Output, C::Error> {
    let original_len = format.len();
    while !format.is_empty() {
        let i = format
            .find(|c: char| c == '%' || c.is_whitespace())
            .unwrap_or(format.len());
        if i > 0 {
            let start = original_len - format.len();
            let (lit, rest) = format.split_at(i);
            collector.literal(lit, start..(start + i))?;
            format = rest;
            if format.is_empty() {
                break;
            }
        }
        if format.starts_with(char::is_whitespace) {
            collector.spaces()?;
            format = format.trim_start();
            continue;
        }
        assert_eq!(format.as_bytes()[0], b'%');
        format = &format[1..];
        if let Some(b) = format.bytes().next() {
            match b {
                b'a' | b'A' => collector.day_of_week_name()?,
                b'b' | b'B' | b'h' => collector.month_name()?,
                b'c' => collector.preferred_date_time()?,
                b'C' => collector.year_prefix()?,
                b'd' | b'e' => collector.day_of_month()?,
                b'D' => collector.date_mmddyy_slash()?,
                b'F' => collector.date_yyyymmdd_hyphen()?,
                b'H' | b'k' => collector.hour_of_day()?,
                b'I' | b'l' => collector.hour_of_day_12()?,
                b'j' => collector.day_of_year()?,
                b'm' => collector.month_of_year()?,
                b'M' => collector.minute_of_hour()?,
                b'n' => collector.new_line()?,
                b'p' | b'P' => collector.ampm()?,
                b'r' => collector.time_ampm()?,
                b'R' => collector.hour_minute_of_day()?,
                b'S' => collector.second_of_minute()?,
                b'f' => collector.nanosecond_of_minute()?,
                b't' => collector.tab()?,
                b'T' => collector.time_of_day()?,
                b'U' => collector.week_number_of_current_year_start_sunday()?,
                b'w' => collector.day_of_week_from_sunday_as_0()?,
                b'W' => collector.week_number_of_current_year_start_monday()?,
                b'x' => collector.preferred_date()?,
                b'X' => collector.preferred_time_of_day()?,
                b'y' => collector.year_suffix()?,
                b'Y' => collector.year()?,
                b'z' => collector.timezone()?,
                b'Z' => collector.timezone_name()?,
                b'%' => collector.percent()?,
                _ => {
                    let c = format.chars().next().unwrap();
                    collector.unknown(c)?;
                    format = &format[c.len_utf8()..];
                    continue;
                }
            }
            format = &format[1..];
        } else {
            collector.percent()?;
        }
    }
    collector.output()
}

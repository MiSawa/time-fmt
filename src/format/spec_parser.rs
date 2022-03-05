use std::slice::SliceIndex;

/// E and O are not implemented.
/// Those require `nl-langinfo` lookup is default-implemented as if it were a POSIX locale.
/// If you'd want to implement it properly, it's your responsibility to recursively parse
/// the format you get from `nl-langinfo`, and prevent infinite recursion.
pub(crate) trait Collector {
    type Output;
    type Error;
    /// `%a`. `nl_langinfo`-dependent.
    fn day_of_week_name_short(&mut self) -> Result<(), Self::Error>;
    /// `%A`. `nl_langinfo`-dependent.
    fn day_of_week_name_long(&mut self) -> Result<(), Self::Error>;
    /// `%b` and `%h`. `nl_langinfo`-dependent.
    fn month_name_short(&mut self) -> Result<(), Self::Error>;
    /// `%B`. `nl_langinfo`-dependent.
    fn month_name_long(&mut self) -> Result<(), Self::Error>;
    /// `%c`. Same as `%a %b %e %T %Y` in POSIX locale. `nl_langinfo`-dependent.
    #[inline]
    fn preferred_date_time(&mut self) -> Result<(), Self::Error> {
        self.day_of_week_name_short()?;
        self.static_str(" ")?;
        self.month_name_short()?;
        self.static_str(" ")?;
        self.day_of_month_blank()?;
        self.static_str(" ")?;
        self.year()?;
        self.static_str(" ")?;
        self.time_of_day()?;
        self.year()
    }
    /// `%C`. `00` to unbounded number.
    fn year_prefix(&mut self) -> Result<(), Self::Error>;
    /// `%d`. `01` to `31`.
    fn day_of_month(&mut self) -> Result<(), Self::Error>;
    /// `%D`. `%m/%d/%y` (American......).
    #[inline]
    fn date_mmddyy_slash(&mut self) -> Result<(), Self::Error> {
        self.month_of_year()?;
        self.static_str("/")?;
        self.day_of_month()?;
        self.static_str("/")?;
        self.year_suffix()
    }
    /// `%e`. ` 1` to `31`.
    fn day_of_month_blank(&mut self) -> Result<(), Self::Error>;
    /// `%F`. `%Y-%m-%d`.
    #[inline]
    fn date_yyyymmdd_hyphen(&mut self) -> Result<(), Self::Error> {
        self.year()?;
        self.static_str("-")?;
        self.month_of_year()?;
        self.static_str("-")?;
        self.day_of_month()
    }
    /// `%g`. ISO 8601 week-based year modulo 100.
    fn iso8601_week_based_year_suffix(&mut self) -> Result<(), Self::Error>;
    /// `%G`. ISO 8601 week-based year.
    fn iso8601_week_based_year(&mut self) -> Result<(), Self::Error>;
    /// `%H`. `00` to `23`.
    fn hour_of_day(&mut self) -> Result<(), Self::Error>;
    /// `%I`. `01` to `12`.
    fn hour_of_day_12(&mut self) -> Result<(), Self::Error>;
    /// `%j`. `001` to `336`.
    fn day_of_year(&mut self) -> Result<(), Self::Error>;
    /// `%k`. ` 0` to `23`.
    fn hour_of_day_blank(&mut self) -> Result<(), Self::Error>;
    /// `%l`. ` 1` to `12`.
    fn hour_of_day_12_blank(&mut self) -> Result<(), Self::Error>;
    /// `%m`. `01` to `12`.
    fn month_of_year(&mut self) -> Result<(), Self::Error>;
    /// `%M`. `00` to `59`.
    fn minute_of_hour(&mut self) -> Result<(), Self::Error>;
    /// `%n`.
    #[inline]
    fn new_line(&mut self) -> Result<(), Self::Error> {
        self.static_str("\n")
    }
    /// `%p`. `AM` or `PM`. `nl_langinfo`-dependent.
    fn ampm(&mut self) -> Result<(), Self::Error>;
    /// `%P`. `am` or `pm`. `nl_langinfo`-dependent.
    fn ampm_lower(&mut self) -> Result<(), Self::Error>;
    /// `%r`. Same as `%I:%M:%S %p` in POSIX locale. `nl_langinfo`-dependent.
    #[inline]
    fn time_ampm(&mut self) -> Result<(), Self::Error> {
        self.hour_of_day_12()?;
        self.static_str(":")?;
        self.minute_of_hour()?;
        self.static_str(":")?;
        self.second_of_minute()?;
        self.static_str(" ")?;
        self.ampm()
    }
    /// `%R`. Same as `%H:%M`.
    #[inline]
    fn hour_minute_of_day(&mut self) -> Result<(), Self::Error> {
        self.hour_of_day()?;
        self.static_str(":")?;
        self.minute_of_hour()
    }
    /// `%S`. `00` to `60`.
    fn second_of_minute(&mut self) -> Result<(), Self::Error>;
    /// `%t`.
    #[inline]
    fn tab(&mut self) -> Result<(), Self::Error> {
        self.static_str("\t")
    }
    /// `%T`. Same as `%H:%M:%S`.
    #[inline]
    fn time_of_day(&mut self) -> Result<(), Self::Error> {
        self.hour_of_day()?;
        self.static_str(":")?;
        self.minute_of_hour()?;
        self.static_str(":")?;
        self.second_of_minute()
    }
    /// `%u`. `1` to `7`
    fn day_of_week_from_monday_as_1(&mut self) -> Result<(), Self::Error>;
    /// `%U`. `00` to `53`.
    fn week_number_of_current_year_start_sunday(&mut self) -> Result<(), Self::Error>;
    /// `%V`. `01` to `53`.
    fn iso8601_week_number(&mut self) -> Result<(), Self::Error>;
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

pub(crate) fn parse_conversion_specifications<C: Collector>(
    mut format: &str,
    mut collector: C,
) -> Result<C::Output, C::Error> {
    let original_len = format.len();
    while !format.is_empty() {
        let i = format
            .bytes()
            .position(|c| c == b'%')
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
        assert_eq!(format.as_bytes()[0], b'%');
        format = &format[1..];
        if let Some(b) = format.bytes().next() {
            match b {
                b'a' => collector.day_of_week_name_short()?,
                b'A' => collector.day_of_week_name_long()?,
                b'b' | b'h' => collector.month_name_short()?,
                b'B' => collector.month_name_long()?,
                b'c' => collector.preferred_date_time()?,
                b'C' => collector.year_prefix()?,
                b'd' => collector.day_of_month()?,
                b'D' => collector.date_mmddyy_slash()?,
                b'e' => collector.day_of_month_blank()?,
                b'F' => collector.date_yyyymmdd_hyphen()?,
                b'g' => collector.iso8601_week_based_year_suffix()?,
                b'G' => collector.iso8601_week_based_year()?,
                b'H' => collector.hour_of_day()?,
                b'I' => collector.hour_of_day_12()?,
                b'j' => collector.day_of_year()?,
                b'k' => collector.hour_of_day_blank()?,
                b'l' => collector.hour_of_day_12_blank()?,
                b'm' => collector.month_of_year()?,
                b'M' => collector.minute_of_hour()?,
                b'n' => collector.new_line()?,
                b'p' => collector.ampm()?,
                b'P' => collector.ampm_lower()?,
                b'r' => collector.time_ampm()?,
                b'R' => collector.hour_minute_of_day()?,
                b'S' => collector.second_of_minute()?,
                b't' => collector.tab()?,
                b'T' => collector.time_of_day()?,
                b'u' => collector.day_of_week_from_monday_as_1()?,
                b'U' => collector.week_number_of_current_year_start_sunday()?,
                b'V' => collector.iso8601_week_number()?,
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

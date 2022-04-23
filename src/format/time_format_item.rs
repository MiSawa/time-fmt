use std::slice::SliceIndex;

use thiserror::Error;
use time::format_description::{modifier, Component, FormatItem};

use super::spec_parser::Collector;

#[derive(Error, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Error {
    #[error("Unknown specifier `%{0}`")]
    UnknownSpecifier(char),
    #[error("No FormatItem that represent {0}")]
    NoCorrespondingFormatItem(&'static str),
}

struct ToFormatItemCollector<'a> {
    fmt: &'a [u8],
    items: Vec<FormatItem<'a>>,
}

impl<'a> ToFormatItemCollector<'a> {
    fn new(fmt: &'a [u8]) -> Self {
        Self {
            fmt,
            items: Default::default(),
        }
    }
}

impl<'a> Collector for ToFormatItemCollector<'a> {
    type Output = Vec<FormatItem<'a>>;
    type Error = Error;

    #[inline]
    fn day_of_week_name_short(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Weekday::default();
        modifier.repr = modifier::WeekdayRepr::Short;
        self.items
            .push(FormatItem::Component(Component::Weekday(modifier)));
        Ok(())
    }

    #[inline]
    fn day_of_week_name_long(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Weekday::default();
        modifier.repr = modifier::WeekdayRepr::Long;
        self.items
            .push(FormatItem::Component(Component::Weekday(modifier)));
        Ok(())
    }

    #[inline]
    fn month_name_short(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Month::default();
        modifier.repr = modifier::MonthRepr::Short;
        self.items
            .push(FormatItem::Component(Component::Month(modifier)));
        Ok(())
    }

    #[inline]
    fn month_name_long(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Month::default();
        modifier.repr = modifier::MonthRepr::Long;
        self.items
            .push(FormatItem::Component(Component::Month(modifier)));
        Ok(())
    }

    #[inline]
    fn year_prefix(&mut self) -> Result<(), Self::Error> {
        Err(Self::Error::NoCorrespondingFormatItem("%C"))
    }

    #[inline]
    fn day_of_month(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Day::default();
        modifier.padding = modifier::Padding::Zero;
        self.items
            .push(FormatItem::Component(Component::Day(modifier)));
        Ok(())
    }

    #[inline]
    fn day_of_month_blank(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Day::default();
        modifier.padding = modifier::Padding::Space;
        self.items
            .push(FormatItem::Component(Component::Day(modifier)));
        Ok(())
    }

    #[inline]
    fn iso8601_week_based_year_suffix(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Year::default();
        modifier.iso_week_based = true;
        modifier.repr = modifier::YearRepr::LastTwo;
        self.items
            .push(FormatItem::Component(Component::Year(modifier)));
        Ok(())
    }

    #[inline]
    fn iso8601_week_based_year(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Year::default();
        modifier.iso_week_based = true;
        self.items
            .push(FormatItem::Component(Component::Year(modifier)));
        Ok(())
    }

    #[inline]
    fn hour_of_day(&mut self) -> Result<(), Self::Error> {
        let modifier = modifier::Hour::default();
        self.items
            .push(FormatItem::Component(Component::Hour(modifier)));
        Ok(())
    }

    #[inline]
    fn hour_of_day_12(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Hour::default();
        modifier.is_12_hour_clock = true;
        self.items
            .push(FormatItem::Component(Component::Hour(modifier)));
        Ok(())
    }

    #[inline]
    fn day_of_year(&mut self) -> Result<(), Self::Error> {
        let modifier = modifier::Ordinal::default();
        self.items
            .push(FormatItem::Component(Component::Ordinal(modifier)));
        Ok(())
    }

    #[inline]
    fn hour_of_day_blank(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Hour::default();
        modifier.padding = modifier::Padding::Space;
        self.items
            .push(FormatItem::Component(Component::Hour(modifier)));
        Ok(())
    }

    #[inline]
    fn hour_of_day_12_blank(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Hour::default();
        modifier.padding = modifier::Padding::Space;
        modifier.is_12_hour_clock = true;
        self.items
            .push(FormatItem::Component(Component::Hour(modifier)));
        Ok(())
    }

    #[inline]
    fn month_of_year(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Month::default();
        modifier.repr = modifier::MonthRepr::Numerical;
        self.items
            .push(FormatItem::Component(Component::Month(modifier)));
        Ok(())
    }

    #[inline]
    fn minute_of_hour(&mut self) -> Result<(), Self::Error> {
        let modifier = modifier::Minute::default();
        self.items
            .push(FormatItem::Component(Component::Minute(modifier)));
        Ok(())
    }

    #[inline]
    fn ampm(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Period::default();
        modifier.is_uppercase = true;
        self.items
            .push(FormatItem::Component(Component::Period(modifier)));
        Ok(())
    }

    #[inline]
    fn ampm_lower(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Period::default();
        modifier.is_uppercase = false;
        self.items
            .push(FormatItem::Component(Component::Period(modifier)));
        Ok(())
    }

    #[inline]
    fn second_of_minute(&mut self) -> Result<(), Self::Error> {
        let modifier = modifier::Second::default();
        self.items
            .push(FormatItem::Component(Component::Second(modifier)));
        Ok(())
    }

    #[inline]
    fn nanosecond_of_minute(&mut self) -> Result<(), Self::Error> {
        let modifier = modifier::Subsecond::default();
        self.items
            .push(FormatItem::Component(Component::Subsecond(modifier)));
        Ok(())
    }

    #[inline]
    fn day_of_week_from_monday_as_1(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Weekday::default();
        modifier.repr = modifier::WeekdayRepr::Monday;
        modifier.one_indexed = true;
        self.items
            .push(FormatItem::Component(Component::Weekday(modifier)));
        Ok(())
    }

    #[inline]
    fn week_number_of_current_year_start_sunday(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::WeekNumber::default();
        modifier.repr = modifier::WeekNumberRepr::Sunday;
        self.items
            .push(FormatItem::Component(Component::WeekNumber(modifier)));
        Ok(())
    }

    #[inline]
    fn iso8601_week_number(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::WeekNumber::default();
        modifier.repr = modifier::WeekNumberRepr::Iso;
        self.items
            .push(FormatItem::Component(Component::WeekNumber(modifier)));
        Ok(())
    }

    #[inline]
    fn day_of_week_from_sunday_as_0(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Weekday::default();
        modifier.repr = modifier::WeekdayRepr::Sunday;
        modifier.one_indexed = false;
        self.items
            .push(FormatItem::Component(Component::Weekday(modifier)));
        Ok(())
    }

    #[inline]
    fn week_number_of_current_year_start_monday(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::WeekNumber::default();
        modifier.repr = modifier::WeekNumberRepr::Monday;
        self.items
            .push(FormatItem::Component(Component::WeekNumber(modifier)));
        Ok(())
    }

    #[inline]
    fn year_suffix(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Year::default();
        modifier.repr = modifier::YearRepr::LastTwo;
        self.items
            .push(FormatItem::Component(Component::Year(modifier)));
        Ok(())
    }

    #[inline]
    fn year(&mut self) -> Result<(), Self::Error> {
        let modifier = modifier::Year::default();
        self.items
            .push(FormatItem::Component(Component::Year(modifier)));
        Ok(())
    }

    #[inline]
    fn timezone(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::OffsetHour::default();
        modifier.sign_is_mandatory = true;
        self.items
            .push(FormatItem::Component(Component::OffsetHour(modifier)));
        let modifier = modifier::OffsetMinute::default();
        self.items
            .push(FormatItem::Component(Component::OffsetMinute(modifier)));
        Ok(())
    }

    #[inline]
    fn timezone_name(&mut self) -> Result<(), Self::Error> {
        Err(Self::Error::NoCorrespondingFormatItem("timezone name"))
    }

    #[inline]
    fn static_str(&mut self, s: &'static str) -> Result<(), Self::Error> {
        self.items.push(FormatItem::Literal(s.as_bytes()));
        Ok(())
    }

    #[inline]
    fn literal(
        &mut self,
        _lit: &str,
        fmt_span: impl SliceIndex<[u8], Output = [u8]>,
    ) -> Result<(), Self::Error> {
        self.items.push(FormatItem::Literal(&self.fmt[fmt_span]));
        Ok(())
    }

    #[inline]
    fn unknown(&mut self, specifier: char) -> Result<(), Self::Error> {
        Err(Self::Error::UnknownSpecifier(specifier))
    }

    #[inline]
    fn output(self) -> Result<Self::Output, Self::Error> {
        Ok(self.items)
    }
}

pub fn parse_to_format_item(fmt: &str) -> Result<Vec<FormatItem>, Error> {
    let collector = ToFormatItemCollector::new(fmt.as_bytes());
    super::spec_parser::parse_conversion_specifications(fmt, collector)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() -> Result<(), super::Error> {
        assert_eq!(
            super::parse_to_format_item("%Y-%m-%d")?,
            super::parse_to_format_item("%F")?,
        );
        Ok(())
    }
}

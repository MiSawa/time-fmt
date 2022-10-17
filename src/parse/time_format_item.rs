use std::slice::SliceIndex;

use thiserror::Error;
use time::format_description::{modifier, Component, FormatItem};

use super::desc_parser::Collector;

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

macro_rules! all_paddings {
    ($ret: expr, $create_base: expr, $component_builder: expr) => {
        const fn with_padding(pad: modifier::Padding) -> Component {
            let mut m = $create_base;
            m.padding = pad;
            $component_builder(m)
        }
        static ITEMS: [FormatItem; 3] = [
            FormatItem::Component(with_padding(modifier::Padding::Zero)),
            FormatItem::Component(with_padding(modifier::Padding::Space)),
            FormatItem::Component(with_padding(modifier::Padding::None)),
        ];
        $ret.push(FormatItem::First(&ITEMS));
    };
}

impl<'a> Collector for ToFormatItemCollector<'a> {
    type Output = Vec<FormatItem<'a>>;
    type Error = Error;

    #[inline]
    fn spaces(&mut self) -> Result<(), Self::Error> {
        // TODO: How to allow more than one?
        self.items.push(FormatItem::Optional(&FormatItem::First(&[
            FormatItem::Literal(b" "),
            FormatItem::Literal(b"\n"),
            FormatItem::Literal(b"\t"),
        ])));
        Ok(())
    }

    #[inline]
    fn day_of_week_name(&mut self) -> Result<(), Self::Error> {
        const fn short() -> FormatItem<'static> {
            let mut short = modifier::Weekday::default();
            short.repr = modifier::WeekdayRepr::Short;
            short.case_sensitive = false;
            FormatItem::Component(Component::Weekday(short))
        }
        const fn long() -> FormatItem<'static> {
            let mut long = modifier::Weekday::default();
            long.repr = modifier::WeekdayRepr::Long;
            long.case_sensitive = false;
            FormatItem::Component(Component::Weekday(long))
        }
        static ITEMS: [FormatItem; 2] = [long(), short()];
        self.items.push(FormatItem::First(&ITEMS));
        Ok(())
    }

    #[inline]
    fn month_name(&mut self) -> Result<(), Self::Error> {
        const fn short() -> FormatItem<'static> {
            let mut short = modifier::Month::default();
            short.repr = modifier::MonthRepr::Short;
            short.case_sensitive = false;
            FormatItem::Component(Component::Month(short))
        }
        const fn long() -> FormatItem<'static> {
            let mut long = modifier::Month::default();
            long.repr = modifier::MonthRepr::Long;
            long.case_sensitive = false;
            FormatItem::Component(Component::Month(long))
        }
        static ITEMS: [FormatItem; 2] = [long(), short()];
        self.items.push(FormatItem::First(&ITEMS));
        Ok(())
    }

    #[inline]
    fn year_prefix(&mut self) -> Result<(), Self::Error> {
        Err(Self::Error::NoCorrespondingFormatItem("%C"))
    }

    #[inline]
    fn day_of_month(&mut self) -> Result<(), Self::Error> {
        all_paddings!(self.items, modifier::Day::default(), Component::Day);
        Ok(())
    }

    #[inline]
    fn hour_of_day(&mut self) -> Result<(), Self::Error> {
        all_paddings!(self.items, modifier::Hour::default(), Component::Hour);
        Ok(())
    }

    #[inline]
    fn hour_of_day_12(&mut self) -> Result<(), Self::Error> {
        all_paddings!(
            self.items,
            {
                let mut base = modifier::Hour::default();
                base.is_12_hour_clock = true;
                base
            },
            Component::Hour
        );
        Ok(())
    }

    #[inline]
    fn day_of_year(&mut self) -> Result<(), Self::Error> {
        all_paddings!(self.items, modifier::Ordinal::default(), Component::Ordinal);
        Ok(())
    }

    #[inline]
    fn month_of_year(&mut self) -> Result<(), Self::Error> {
        all_paddings!(self.items, modifier::Month::default(), Component::Month);
        Ok(())
    }

    #[inline]
    fn minute_of_hour(&mut self) -> Result<(), Self::Error> {
        all_paddings!(self.items, modifier::Minute::default(), Component::Minute);
        Ok(())
    }

    #[inline]
    fn ampm(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::Period::default();
        modifier.case_sensitive = false;
        self.items
            .push(FormatItem::Component(Component::Period(modifier)));
        Ok(())
    }

    #[inline]
    fn second_of_minute(&mut self) -> Result<(), Self::Error> {
        all_paddings!(self.items, modifier::Second::default(), Component::Second);
        Ok(())
    }

    #[inline]
    fn nanosecond_of_second(&mut self) -> Result<(), Self::Error> {
        let modifier = modifier::Subsecond::default();
        self.items
            .push(FormatItem::Component(Component::Subsecond(modifier)));
        Ok(())
    }

    #[inline]
    fn tab(&mut self) -> Result<(), Self::Error> {
        self.spaces()
    }

    #[inline]
    fn week_number_of_current_year_start_sunday(&mut self) -> Result<(), Self::Error> {
        all_paddings!(
            self.items,
            {
                let mut base = modifier::WeekNumber::default();
                base.repr = modifier::WeekNumberRepr::Sunday;
                base
            },
            Component::WeekNumber
        );
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
        all_paddings!(
            self.items,
            {
                let mut base = modifier::WeekNumber::default();
                base.repr = modifier::WeekNumberRepr::Monday;
                base
            },
            Component::WeekNumber
        );
        Ok(())
    }

    #[inline]
    fn year_suffix(&mut self) -> Result<(), Self::Error> {
        all_paddings!(
            self.items,
            {
                let mut base = modifier::Year::default();
                base.repr = modifier::YearRepr::LastTwo;
                base
            },
            Component::Year
        );
        Ok(())
    }

    #[inline]
    fn year(&mut self) -> Result<(), Self::Error> {
        all_paddings!(self.items, modifier::Year::default(), Component::Year);
        Ok(())
    }

    #[inline]
    fn timezone(&mut self) -> Result<(), Self::Error> {
        let mut modifier = modifier::OffsetHour::default();
        modifier.sign_is_mandatory = true;
        self.items
            .push(FormatItem::Component(Component::OffsetHour(modifier)));
        self.items
            .push(FormatItem::Optional(&FormatItem::Literal(b":")));
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
    fn unconsumed_input(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn output(self) -> Result<Self::Output, Self::Error> {
        Ok(self.items)
    }
}

pub fn parse_to_format_item(fmt: &str) -> Result<Vec<FormatItem>, Error> {
    let collector = ToFormatItemCollector::new(fmt.as_bytes());
    super::desc_parser::parse_format_specifications(fmt, collector, false)
}

#[cfg(test)]
mod tests {
    use time::{macros::datetime, OffsetDateTime, PrimitiveDateTime};

    use super::parse_to_format_item;

    #[test]
    fn it_works() -> Result<(), super::Error> {
        assert_eq!(
            parse_to_format_item("%Y-%m-%d")?,
            parse_to_format_item("%F")?,
        );
        Ok(())
    }

    #[test]
    fn parse_primitive_datetime() -> Result<(), Box<dyn std::error::Error>> {
        let format_items = parse_to_format_item("%Y-%m-%d %H:%M:%S")?;
        assert_eq!(
            PrimitiveDateTime::parse("2012-05-21 12:09:14", &format_items)?,
            datetime!(2012-05-21 12:09:14)
        );
        Ok(())
    }

    #[test]
    fn parse_offset_datetime() -> Result<(), Box<dyn std::error::Error>> {
        let format_items = parse_to_format_item("%Y-%m-%d %H:%M:%S %z")?;
        assert_eq!(
            OffsetDateTime::parse("2012-05-21 12:09:14 +0900", &format_items)?,
            datetime!(2012-05-21 12:09:14 +9:00)
        );
        assert_eq!(
            OffsetDateTime::parse("2012-05-21 12:09:14 +09:00", &format_items)?,
            datetime!(2012-05-21 12:09:14 +9:00)
        );
        Ok(())
    }
}

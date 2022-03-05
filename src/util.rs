use time::{Month, Weekday};

const MONTH_LONG: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const MONTH_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const MONTHS0: [Month; 12] = [
    Month::January,
    Month::February,
    Month::March,
    Month::April,
    Month::May,
    Month::June,
    Month::July,
    Month::August,
    Month::September,
    Month::October,
    Month::November,
    Month::December,
];
const WEEKDAY_LONG: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];
const WEEKDAY_SHORT: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const AMPM_UPPER: [&str; 2] = ["AM", "PM"];
const AMPM_LOWER: [&str; 2] = ["am", "pm"];

#[inline]
pub(crate) const fn ampm_upper(hour: u8) -> &'static str {
    AMPM_UPPER[if hour < 12 { 0 } else { 1 }]
}
#[inline]
pub(crate) const fn ampm_lower(hour: u8) -> &'static str {
    AMPM_LOWER[if hour < 12 { 0 } else { 1 }]
}
#[inline]
pub(crate) const fn month_long_str(month: Month) -> &'static str {
    MONTH_LONG[(month as u8 - 1) as usize]
}
#[inline]
pub(crate) const fn month_short_str(month: Month) -> &'static str {
    MONTH_SHORT[(month as u8 - 1) as usize]
}
#[inline]
pub(crate) const fn weekday_long_str(weekday: Weekday) -> &'static str {
    WEEKDAY_LONG[weekday as u8 as usize]
}
#[inline]
pub(crate) const fn weekday_short_str(weekday: Weekday) -> &'static str {
    WEEKDAY_SHORT[weekday as u8 as usize]
}
#[inline]
pub(crate) const fn get_month(month: u8) -> Option<Month> {
    // TODO: Use MONTHD0.get((month - 1) as usize).copied() once copied() get to a const fn in
    // stable.
    if 1 <= month && month <= 12 {
        Some(MONTHS0[(month - 1) as usize])
    } else {
        None
    }
}

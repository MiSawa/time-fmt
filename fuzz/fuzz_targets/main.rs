#![no_main]

use libfuzzer_sys::fuzz_target;
use time::OffsetDateTime;
use time_fmt::{format::format_offset_date_time, parse::parse_date_time_maybe_with_zone};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
enum Query {
    Format { fmt: String, timestamp: i128 },
    Parse { fmt: String, input: String },
}

fuzz_target!(|query: Query| {
    match query {
        Query::Format { ref fmt, timestamp } => {
            if let Ok(datetime) = OffsetDateTime::from_unix_timestamp_nanos(timestamp) {
                format_offset_date_time(fmt, datetime).ok();
            }
        }
        Query::Parse { ref fmt, ref input } => {
            parse_date_time_maybe_with_zone(fmt, input).ok();
        }
    }
});

[![CI](https://github.com/MiSawa/time-fmt/actions/workflows/ci.yml/badge.svg)](https://github.com/MiSawa/time-fmt/actions/workflows/ci.yml) [![crates.io](https://img.shields.io/crates/v/time-fmt.svg)](https://crates.io/crates/time-fmt)

# time-fmt

This is a library that formats/parses datetime of the [time crate](https://github.com/time-rs/time) with _somewhat_ more `strftime`/`strptime`-compatible format specification.

**You should really use the format of the time crate**, if there's no strong reason not to.

## Features

- ANSI/ISO C `strftime`-like function.
  - `%C`, `%d`, `%D`, `%e`, `%F`, `%g`, `%G`, `%h`, `%H`, `%I`, `%j`, `%k`, `%l`, `%m`, `%M`, `%n`, `%R`, `%S`, `%t`, `%T`, `%u`, `%U`, `%V`, `%w`, `%W`, `%y`, `%Y`, `%%`.
  - Those treated as if you were in C/POSIX locale: `%a`, `%A`, `%b`, `%B`, `%c`, `%p`, `%P`, `%r`, `%x`, `%X`.
  - Available-ish (see: incompatibilities section): `%z`, `%Z`.
- POSIX C `strptime`-like function.
  - `%C`, `%d`, `%D`, `%e`, `%F`, `%h`, `%H`, `%I`, `%j`, `%k`, `%l`, `%m`, `%M`, `%n`, `%R`, `%S`, `%t`, `%T`, `%W`, `%y`, `%Y`, `%%`.
  - Those treated as if you were in C/POSIX locale: `%b`, `%B`, `%c`, `%p`, `%P`, `%r`, `%x`, `%X`
  - Available-ish but the way handling the parsed value is delegated to the user: `%z`, `%Z`.
  - Those parsed (as if you were in C/POSIX locale) but ignored: `%a`, `%A`, `%U`, `%w`.
- A function that converts `strftime`-like conversion specification to `Vec<FormatItem>` of the time crate.
- A function that converts `strptime`-like conversion description to `Vec<FormatItem>` of the time crate.

## *Non*-features ...yet. Contributions are welcomed!

- Not well-tested. Obviously require a lot more tests.
- Compiling format specifications to an intermediate representation is unsupported. Convert them to `Vec<FormatItem>` instead.
- `%E*` and `%O*` should be implemented as if it were in the C/POSIX locale; i.e. fall back to the normal ones.
- Minimum field width (should be applicable to `C`, `F`, `G`, `Y`) and flags (`0`, `+`).

## Incompatibilities / Cautions

- `strftime`-like ones
  - Those require `nl_langinfo` lookups, namely `%a`, `%A`, `%b`, `%h`, `%B`, `%c`, `%p`, `%P`, `%r`, `%x`, and `%X` are not implemented to do so. Instead, they are hardcoded to use that of C/POSIX locale.
  - Era-based formats, namely those starts with `%E` are unsupported.
  - Alternative numeric symbols, namely those starts with `%O` are unsupported.
  - `%z` doesn't work if you passed `PrimitiveDateTime`. It'll be substituted to the empty string, as if "no time zone is determinable".
  - `%Z` works only if you used a function that takes a zone name. Otherwise substituted to the empty string, as if "no time zone is determinable".
  - `%` followed by a character that doesn't compose a conversion specifier that *we* support will result into an error.
- `strptime`-like ones
  - Years has to fit in 4 bytes, i.e. before the year -999 or after the year 9999 are unsupported.
  - Day of week specifiers, week of year specifiers, namely `%a`, `%A`, `%U`, and `%w` are matched to the input but ignored.
  - Since our structure that represents date/time are not something like `struct tm` of C language, inconsistent input will result in an unspecified behavior.
    - For example, one can specify the month, the day of the month, and the day of the year. But it's unclear what to do if the day of the year doesn't match what (month, day of the month) pair says. Currently it choose what day of the year says, it may be changed to do something else, for example returning `Result::Err` in a future release *without bumping the major version*.
  - Offset/timezone info given by `%z`, `%Z` are not refleted to the returned date time. Instead, we return a pair of `PrimitiveDateTime` and the parsed offset / timezone name.
    - If `Z` was given for `%z`, it will be treated as same as `+00:00`.
- Convertion from `strftime`-like conversion specification to `Vec<FormatItem>`
  - `%C` (century) and `%Z` (timezone name) are unsupported as no corresponding `FormatItem` exists.
- Convertion from `strptime`-like conversion description to `Vec<FormatItem>`
  - `%C` (century) and `%Z` (timezone name) are unsupported as no corresponding `FormatItem` exists.
  - `%z` (timezone specifier) can't be `Z`, as we can't seem to do it with `FormatItem`.
  - Whitespaces won't eat more than one white space, as there's no repetitions in `FormatItem`.


## Examples

```rust
use time::{macros::{datetime, offset}, UtcOffset};
use time_fmt::{format::*, parse::*};

let dt = datetime!(2022-03-06 12:34:56);

// Format primitive date time
assert_eq!(
    format_date_time("%Y-%m-%d %H:%M:%S", dt).unwrap(),
    "2022-03-06 12:34:56"
);
// Format offset date time
assert_eq!(
    format_offset_date_time("%Y-%m-%d %H:%M:%S %z", dt.assume_offset(offset!(+9:00))).unwrap(),
    "2022-03-06 12:34:56 +0900"
);
// With timezone_name feature
assert_eq!(
    format_zoned_date_time("%Y-%m-%d %H:%M:%S %Z", dt, offset!(+9:00), "JST").unwrap(),
    "2022-03-06 12:34:56 JST"
);
// Parse date time
assert_eq!(
    parse_date_time_maybe_with_zone("%Y-%m-%d %H:%M:%S %z", "2022-03-06 12:34:56 +0900")
    .unwrap(),
    (
        dt,
        Some(TimeZoneSpecifier::Offset(
                UtcOffset::from_hms(9, 0, 0).unwrap()
        ))
    )
);
```


## Publish new version

Note for myself.

```shell
$ git switch master                 # make sure you're on the master branch
$ cargo release patch               # to dry-run the release
$ cargo release patch --execute     # to actually execute the release
```

## License

Licensed under either of

 - Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 - MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed under the terms of both the Apache License, Version 2.0 and the MIT license without any additional terms or conditions.


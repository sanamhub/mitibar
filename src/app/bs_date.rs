//! Conversions between a jiff civil date and a BS date (ADR-0014). The published `nepdate` builds
//! its `jiff` feature on another jiff major, so the app converts through the AD year, month and
//! day.

use jiff::civil::Date;
use nepdate::{Error, NepaliDate};

/// The BS date of the AD date `date`.
///
/// # Errors
///
/// [`Error::OutOfRange`] outside the range `nepdate` supports.
pub fn from_civil(date: Date) -> Result<NepaliDate, Error> {
    // jiff months are 1 to 12 and days 1 to 31, so both fit a u8.
    let month = u8::try_from(date.month()).map_err(|_| Error::InvalidGregorian)?;
    let day = u8::try_from(date.day()).map_err(|_| Error::InvalidGregorian)?;
    NepaliDate::from_gregorian(i32::from(date.year()), month, day)
}

/// The AD date of the BS date `date`.
pub fn to_civil(date: NepaliDate) -> Date {
    let ad = date.to_gregorian();
    // Every supported BS date is inside jiff's years -9999 to 9999, so the fallback is never
    // taken. It keeps the function free of panics.
    let parts = (
        i16::try_from(ad.year()),
        i8::try_from(ad.month()),
        i8::try_from(ad.day()),
    );
    match parts {
        (Ok(y), Ok(m), Ok(d)) => Date::new(y, m, d).unwrap_or(Date::MIN),
        _ => Date::MIN,
    }
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    #[test]
    fn converts_a_date_inside_the_range() {
        assert_eq!(from_civil(date(2024, 7, 30)), NepaliDate::new(2081, 4, 15));
        assert_eq!(from_civil(date(2023, 8, 30)), NepaliDate::new(2080, 5, 13));
    }

    #[test]
    fn edges_of_the_range() {
        assert_eq!(from_civil(date(1844, 4, 11)), Ok(NepaliDate::MIN));
        assert_eq!(from_civil(date(1844, 4, 10)), Err(Error::OutOfRange));
        assert_eq!(from_civil(date(9999, 12, 31)), Err(Error::OutOfRange));
    }

    #[test]
    fn to_civil_round_trips() {
        assert_eq!(to_civil(NepaliDate::MIN), date(1844, 4, 11));
        let bs = NepaliDate::new(2081, 4, 15).expect("BS 2081-04-15");
        assert_eq!(to_civil(bs), date(2024, 7, 30));
        assert_eq!(from_civil(to_civil(NepaliDate::MAX)), Ok(NepaliDate::MAX));
    }
}

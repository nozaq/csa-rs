use nom::bytes::complete::{tag, take};
use nom::character::complete::digit1;
use nom::combinator::{map_res, opt, verify};
use nom::sequence::preceded;
use nom::*;
use std::convert::TryFrom;
use std::str;
use std::time::Duration;
use time::{Date as NativeDate, Month, Time as NativeTime};

use crate::value::{Time, TimeLimit};

fn take_2_digits(input: &[u8]) -> IResult<&[u8], i32> {
    map_res(map_res(take(2usize), str::from_utf8), |s: &str| s.parse())(input)
}

fn take_4_digits(input: &[u8]) -> IResult<&[u8], i32> {
    map_res(map_res(take(4usize), str::from_utf8), |s: &str| s.parse())(input)
}

fn take_n_digits(input: &[u8]) -> IResult<&[u8], i32> {
    map_res(map_res(digit1, str::from_utf8), |s: &str| s.parse())(input)
}

fn year(input: &[u8]) -> IResult<&[u8], i32> {
    take_4_digits(input)
}

fn month(input: &[u8]) -> IResult<&[u8], i32> {
    verify(take_2_digits, |&d| d > 0 && d < 13)(input)
}

fn day(input: &[u8]) -> IResult<&[u8], i32> {
    verify(take_2_digits, |&d| d > 0 && d < 32)(input)
}

fn hour(input: &[u8]) -> IResult<&[u8], i32> {
    verify(take_2_digits, |&d| (0..24).contains(&d))(input)
}

fn minutes(input: &[u8]) -> IResult<&[u8], i32> {
    verify(take_2_digits, |&d| (0..60).contains(&d))(input)
}

fn seconds(input: &[u8]) -> IResult<&[u8], i32> {
    verify(take_2_digits, |&d| (0..60).contains(&d))(input)
}

fn date(input: &[u8]) -> IResult<&[u8], NativeDate> {
    let (input, year) = year(input)?;
    let (input, _) = tag("/")(input)?;
    let (input, month) = month(input)?;
    let (input, _) = tag("/")(input)?;
    let (input, day) = day(input)?;

    let month = Month::try_from(month as u8).unwrap();
    let date = NativeDate::from_calendar_date(year, month, day as u8).unwrap();

    Ok((input, date))
}

fn time(input: &[u8]) -> IResult<&[u8], NativeTime> {
    let (input, hour) = hour(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, minutes) = minutes(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, seconds) = seconds(input)?;

    Ok((
        input,
        time::Time::from_hms(hour as u8, minutes as u8, seconds as u8).unwrap(),
    ))
}

pub fn datetime(input: &[u8]) -> IResult<&[u8], Time> {
    let (input, date) = date(input)?;
    let (input, time) = opt(preceded(tag(" "), time))(input)?;

    Ok((input, Time { date, time }))
}

pub fn timelimit(input: &[u8]) -> IResult<&[u8], TimeLimit> {
    let (input, hour) = take_n_digits(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, minutes) = minutes(input)?;
    let (input, _) = tag("+")(input)?;
    let (input, byoyomi) = take_n_digits(input)?;

    Ok((
        input,
        TimeLimit {
            main_time: Duration::from_secs(hour as u64 * 60 * 60 + minutes as u64 * 60),
            byoyomi: Duration::from_secs(byoyomi as u64),
        },
    ))
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_year() {
        assert_eq!(year(b"0001"), Result::Ok((&b""[..], 1)));
        assert_eq!(year(b"9999"), Result::Ok((&b""[..], 9999)));
    }

    #[test]
    fn parse_month() {
        assert!(month(b"00").is_err());
        assert_eq!(month(b"01"), Result::Ok((&b""[..], 1)));
        assert_eq!(month(b"12"), Result::Ok((&b""[..], 12)));
        assert!(month(b"13").is_err());
    }

    #[test]
    fn parse_day() {
        assert!(day(b"00").is_err());
        assert_eq!(day(b"01"), Result::Ok((&b""[..], 1)));
        assert_eq!(day(b"31"), Result::Ok((&b""[..], 31)));
        assert!(day(b"32").is_err());
    }

    #[test]
    fn parse_hour() {
        assert_eq!(hour(b"00"), Result::Ok((&b""[..], 0)));
        assert_eq!(hour(b"01"), Result::Ok((&b""[..], 1)));
        assert_eq!(hour(b"23"), Result::Ok((&b""[..], 23)));
        assert!(hour(b"25").is_err());
    }

    #[test]
    fn parse_minutes() {
        assert_eq!(minutes(b"00"), Result::Ok((&b""[..], 0)));
        assert_eq!(minutes(b"01"), Result::Ok((&b""[..], 1)));
        assert_eq!(minutes(b"59"), Result::Ok((&b""[..], 59)));
        assert!(minutes(b"60").is_err());
    }

    #[test]
    fn parse_seconds() {
        assert_eq!(seconds(b"00"), Result::Ok((&b""[..], 0)));
        assert_eq!(seconds(b"01"), Result::Ok((&b""[..], 1)));
        assert_eq!(seconds(b"59"), Result::Ok((&b""[..], 59)));
        assert!(seconds(b"60").is_err());
    }

    #[test]
    fn parse_date() {
        assert_eq!(
            date(b"2002/01/01"),
            Result::Ok((
                &b""[..],
                NativeDate::from_calendar_date(2002, Month::January, 1).unwrap()
            ))
        );
    }

    #[test]
    fn parse_time() {
        assert_eq!(
            time(b"19:00:00"),
            Result::Ok((&b""[..], NativeTime::from_hms(19, 0, 0).unwrap()))
        );
    }

    #[test]
    fn parse_datetime() {
        assert_eq!(
            datetime(b"2002/01/01"),
            Result::Ok((
                &b""[..],
                Time {
                    date: NativeDate::from_calendar_date(2002, Month::January, 1).unwrap(),
                    time: None
                }
            ))
        );
        assert_eq!(
            datetime(b"2002/01/01 19:00:00"),
            Result::Ok((
                &b""[..],
                Time {
                    date: NativeDate::from_calendar_date(2002, Month::January, 1).unwrap(),
                    time: Some(NativeTime::from_hms(19, 0, 0).unwrap())
                }
            ))
        );
    }

    #[test]
    fn parse_timelimit() {
        assert_eq!(
            timelimit(b"00:25+00"),
            Result::Ok((
                &b""[..],
                TimeLimit {
                    main_time: Duration::from_secs(25 * 60),
                    byoyomi: Duration::from_secs(0)
                }
            ))
        );
        assert_eq!(
            timelimit(b"00:30+30"),
            Result::Ok((
                &b""[..],
                TimeLimit {
                    main_time: Duration::from_secs(30 * 60),
                    byoyomi: Duration::from_secs(30)
                }
            ))
        );
        assert_eq!(
            timelimit(b"00:00+30"),
            Result::Ok((
                &b""[..],
                TimeLimit {
                    main_time: Duration::from_secs(0),
                    byoyomi: Duration::from_secs(30)
                }
            ))
        );
    }
}

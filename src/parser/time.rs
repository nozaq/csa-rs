use chrono::{NaiveDate, NaiveTime};
use nom::*;
use std::str;
use std::time::Duration;

use crate::value::{Time, TimeLimit};

named!(
    take_2_digits<i32>,
    map_res!(map_res!(take!(2), |s| str::from_utf8(s)), |s: &str| s
        .parse())
);
named!(
    take_4_digits<i32>,
    map_res!(map_res!(take!(4), |s| str::from_utf8(s)), |s: &str| s
        .parse())
);
named!(
    take_n_digits<i32>,
    map_res!(map_res!(digit, |s| str::from_utf8(s)), |s: &str| s.parse())
);

named!(year<i32>, call!(take_4_digits));
named!(month<i32>, verify!(take_2_digits, |d| d > 0 && d < 13));
named!(day<i32>, verify!(take_2_digits, |d| d > 0 && d < 32));
named!(hour<i32>, verify!(take_2_digits, |d| d >= 0 && d < 24));
named!(minutes<i32>, verify!(take_2_digits, |d| d >= 0 && d < 60));
named!(seconds<i32>, verify!(take_2_digits, |d| d >= 0 && d < 60));

named!(
    date<NaiveDate>,
    do_parse!(
        year: year
            >> tag!("/")
            >> month: month
            >> tag!("/")
            >> day: day
            >> (NaiveDate::from_ymd(year, month as u32, day as u32))
    )
);

named!(
    time<NaiveTime>,
    do_parse!(
        hour: hour
            >> tag!(":")
            >> minutes: minutes
            >> tag!(":")
            >> seconds: seconds
            >> (NaiveTime::from_hms(hour as u32, minutes as u32, seconds as u32))
    )
);

named!(pub datetime<Time>, do_parse!(
    date: date >>
    time: opt!(complete!(preceded!(tag!(" "), time))) >>
    ( Time{date, time} )
));

named!(pub timelimit<TimeLimit>, do_parse!(
    hour: take_n_digits >>
    tag!(":") >>
    minutes: minutes >>
    tag!("+") >>
    byoyomi: take_n_digits >>
    ( TimeLimit {
        main_time: Duration::from_secs(hour as u64 * 60 * 60 + minutes as u64 * 60),
        byoyomi: Duration::from_secs(byoyomi as u64)
    } )
));

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
            Result::Ok((&b""[..], NaiveDate::from_ymd(2002, 1, 1)))
        );
    }

    #[test]
    fn parse_time() {
        assert_eq!(
            time(b"19:00:00"),
            Result::Ok((&b""[..], NaiveTime::from_hms(19, 0, 0)))
        );
    }

    #[test]
    fn parse_datetime() {
        assert_eq!(
            datetime(b"2002/01/01"),
            Result::Ok((
                &b""[..],
                Time {
                    date: NaiveDate::from_ymd(2002, 1, 1),
                    time: None
                }
            ))
        );
        assert_eq!(
            datetime(b"2002/01/01 19:00:00"),
            Result::Ok((
                &b""[..],
                Time {
                    date: NaiveDate::from_ymd(2002, 1, 1),
                    time: Some(NaiveTime::from_hms(19, 0, 0))
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

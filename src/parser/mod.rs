mod game;
mod time;

use nom::ErrorKind;
use std::error::Error;
use std::fmt;

use self::game::game_record;
use crate::value::GameRecord;

#[derive(Debug)]
pub enum CsaError {
    ParseError(ErrorKind),
}

impl fmt::Display for CsaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CsaError::ParseError(ref err) => write!(f, "failed to parse: {}", err.description()),
        }
    }
}

impl Error for CsaError {
    fn description(&self) -> &str {
        match *self {
            CsaError::ParseError(_) => "failed to parse the csa content",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            CsaError::ParseError(ref err) => Some(err),
        }
    }
}

impl From<ErrorKind> for CsaError {
    fn from(err: ErrorKind) -> CsaError {
        CsaError::ParseError(err)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn parse_csa(s: &str) -> Result<GameRecord, CsaError> {
    let record = game_record(s.as_bytes()).to_result()?;

    Ok(record)
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::{self, File};
    use std::io::Read;
    use std::path::Path;

    #[test]
    fn load_fixtures() {
        let fixtures_dir = Path::new("fixtures/");
        let dir_entries = fs::read_dir(fixtures_dir).unwrap();

        for entry in dir_entries {
            let path = entry.unwrap().path();
            if !path.is_file() {
                continue;
            }

            let mut file = File::open(&path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .expect("failed to load a fixuture content");
            let res = parse_csa(&contents);

            assert_eq!(res.is_ok(), true);
        }
    }
}

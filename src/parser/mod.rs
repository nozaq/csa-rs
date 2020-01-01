mod game;
mod time;

use std::error::Error;
use std::fmt;

use self::game::game_record;
use crate::value::GameRecord;

#[derive(Debug)]
pub enum CsaError {
    ParseError(),
}

impl fmt::Display for CsaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CsaError::ParseError() => write!(f, "failed to parse"),
        }
    }
}

impl Error for CsaError {}

////////////////////////////////////////////////////////////////////////////////

pub fn parse_csa(s: &str) -> Result<GameRecord, CsaError> {
    if let Ok((_, record)) = game_record(s.as_bytes()) {
        Ok(record)
    } else {
        Err(CsaError::ParseError())
    }
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

use irc::error::IrcError;

pub use self::Error::*;

pub type Outcome<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Unknown,
    Unauthorized,
    InvalidArgs,
    NoResults,
    ParseErr(failure::Error),
    Ambiguous(i64, Vec<String>),
    IrcErr(Box<IrcError>),
    Throw(failure::Error)
}
impl From<std::num::ParseIntError> for Error {
    fn from(_: std::num::ParseIntError) -> Self {
        NoResults
    }
}
impl From<reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Self {
        NoResults
    }
}
impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        NoResults
    }
}
impl From<getopts::Fail> for Error {
    fn from(_: getopts::Fail) -> Self {
        InvalidArgs
    }
}
impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        use diesel::result::Error::*;
        match e {
            NotFound => NoResults,
            _        => Throw(failure::Error::from(e))
        }
    }
}
impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Self {
        ParseErr(failure::Error::from(e))
    }
}
impl From<chrono::format::ParseError> for Error {
    fn from(e: chrono::format::ParseError) -> Self {
        ParseErr(failure::Error::from(e))
    }
}
impl From<IrcError> for Error {
    fn from(e: IrcError) -> Self {
        IrcErr(Box::new(e))
    }
}
impl From<std::time::SystemTimeError> for Error {
    fn from(e: std::time::SystemTimeError) -> Self {
        Throw(failure::Error::from(e))
    }
}

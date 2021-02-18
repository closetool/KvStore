use {
    failure::{Error, Fail},
    std::result,
};

pub type Result<T> = result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum KvsError {
    #[fail(display = "no such key {}", _0)]
    Get(String),
    #[fail(display = "no such key {}", _0)]
    Remove(String),
    #[fail(display = "no such operatioin {}", _0)]
    UnKnownOperation(String),
    #[fail(display = "no such log file {}", _0)]
    UnKnownLog(u64),
}

use std::result::Result as StdResult;
use failure::Error;

pub type Result<T> = StdResult<T, Error>;
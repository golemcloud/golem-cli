use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

// NonSuccessfulExit is used to signal that an error got resolved with hints or error messages
// already on the command line, thus nothing should be printed in the main error handler,
// but should return non-successful exit code from the process.
#[derive(Debug)]
pub struct NonSuccessfulExit;

impl Display for NonSuccessfulExit {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        //NOP
        Ok(())
    }
}

impl Error for NonSuccessfulExit {}

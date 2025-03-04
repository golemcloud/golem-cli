use crate::model::{ComponentName, WorkerName};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

// Errors that can be enriched with dynamic hints
#[derive(Debug)]
pub enum HintError {
    NoApplicationManifestsFound,
    ComponentNotFound(ComponentName),
    WorkerNotFound(WorkerName),
}

impl Display for HintError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for HintError {}

// Used to signal that an error (usually HintError) got resolved with hints or error messages
// already on the console, thus nothing should be printed in the main error handler,
// but should return non-successful exit code from the process
#[derive(Debug)]
pub struct HandledError;

impl Display for HandledError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        //NOP
        Ok(())
    }
}

impl Error for HandledError {}

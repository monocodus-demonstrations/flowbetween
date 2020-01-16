use std::fmt;
use std::fmt::{Display, Formatter};

///
/// The errors that can be generated by a command
///
#[derive(Clone, Debug, PartialEq)]
pub enum CommandError {
    /// An animation could not be opened
    CouldNotOpenAnimation(String),

    /// An animation could not be created
    CouldNotCreateAnimation(String)
}

impl Display for CommandError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        use self::CommandError::*;

        match self {
            CouldNotOpenAnimation(name)     => write!(fmt, "Could not open animation '{}'", name),
            CouldNotCreateAnimation(name)   => write!(fmt, "Coult not create animation '{}'", name)
        }
    }
}
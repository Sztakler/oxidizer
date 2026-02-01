pub mod error;
pub mod io;
pub mod processor;

pub use error::{OxidizerError, Result};
pub use processor::{Algorithm, Oxidizer};

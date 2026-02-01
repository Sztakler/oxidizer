//! # Oxidizer
//!
//! An audio transformation library that applies "oxidation" effects
//! through low-pass filtering and applying various noise textures.

pub mod error;
pub mod io;
pub mod processor;

pub use error::{OxidizerError, Result};
pub use processor::{OxidationLevel, Oxidizer};

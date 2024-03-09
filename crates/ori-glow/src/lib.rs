#![warn(missing_docs)]

//! A renderer using [`glow`].

mod mesh;
mod render;

pub use render::*;

use std::fmt::Display;

/// An error that can occur when rendering.
#[derive(Debug)]
pub enum GlowError {
    /// Failed to create a surface.
    #[cfg(feature = "glutin")]
    Glutin(glutin::error::Error),
    /// No compatible config found.
    ConfigNotFound,
    /// Failed to request a device.
    Gl(String),
}

#[cfg(feature = "glutin")]
impl From<glutin::error::Error> for GlowError {
    fn from(err: glutin::error::Error) -> Self {
        Self::Glutin(err)
    }
}

impl Display for GlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "glutin")]
            GlowError::Glutin(err) => write!(f, "{}", err),
            GlowError::ConfigNotFound => write!(f, "No compatible config found"),
            GlowError::Gl(err) => write!(f, "{}", err),
        }
    }
}

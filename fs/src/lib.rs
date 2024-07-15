#![no_std]

#[cfg(feature = "fatfs")]
pub mod fatfs;

pub mod error;
pub mod io;

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod buffer;
pub mod storage;

pub mod io;
pub mod sample;

pub mod fmt;
pub mod identifier;

#[cfg(feature = "std")]
pub mod collections;

pub use midi_consts as midi;

pub mod prelude {
    pub use crate::{
        io::{self, Reader, ReaderExt, Writer, WriterExt},
        sample::{Primitive, PrimitiveCast, Sample},
    };
}

//! This library acts as a toolbox on top of [FutureSDR][`futuresdr`] to easily build your own flowgraph.
//! It is made by the community for the community.

// #![feature(async_fn_in_trait)]

#[macro_use]
pub extern crate async_trait;
pub mod byte;
#[cfg(feature = "crossbeam")]
pub mod channel;

#[cfg(feature = "cw")]
pub mod cw;
pub mod filters;
pub mod math;
pub mod octave;
pub mod packet;
pub mod sigmf;
pub mod stdinout;
pub mod stream;
pub mod synchronizers;
pub mod type_converters;
pub mod serde_pmt;

pub use futuresdr;

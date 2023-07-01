//! This library acts as a toolbox on top of [FutureSDR][`futuresdr`] to easily build your own flowgraph.
//! It is made by the community for the community.
#![feature(test)]
#![feature(portable_simd)]

#[macro_use]
pub extern crate async_trait;

pub mod math;
pub mod stdinout;
pub mod stream;
pub mod type_converters;

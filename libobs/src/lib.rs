#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]
#![allow(clippy::approx_constant)]
#![allow(clippy::unreadable_literal)]
#![allow(rustdoc::bare_urls)]
//! # LibOBS bindings (and wrapper) for rust
//! This crate provides bindings to the [LibOBS](https://obsproject.com/) library for rust.
//! Furthermore, this crate provides a safe wrapper around the unsafe functions, which can be found in the [`wrapper`](module@wrapper) module.

#[cfg(test)]
mod test;

mod bindings {
    include!("bindings.rs");
}

pub use bindings::*;

#[cfg(feature="debug-tracing")]
include!(concat!(env!("OUT_DIR"), "/bindings_wrapper.rs"));
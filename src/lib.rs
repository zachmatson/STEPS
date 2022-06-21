//! A simulation of bacterial evolution written by Devin Lake and Zachary Matson
//!
//! Simulates using two-stage doubling procedure in which new mutants can be added in between transfers
//!
//! [Find this project on GitHub](https://github.com/zachmatson/STEPS)

#![warn(missing_docs)]
#![deny(clippy::wildcard_imports)]

pub mod cfg;
pub mod io;
pub mod sim;

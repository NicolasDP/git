/*! Git library in pure rust

[![Build Status](https://travis-ci.org/NicolasDP/git.png?branch=master)](https://travis-ci.org/NicolasDP/git)
*/

#[cfg(test)]
extern crate rustc_serialize;

#[macro_use]
extern crate nom;

#[macro_use]
mod error;
pub mod protocol;
pub mod object;
pub mod refs;
pub mod fs;

pub use error::{Result, GitError};

//! Domain types and services for reading a vibe-doc document tree.
//!
//! This crate deliberately has no HTTP or UI dependency.  It is shared by the
//! command-line application and the future JSON API.

pub mod document;
pub mod index;
pub mod links;
pub mod lint;
pub mod next_index;
pub mod parser;

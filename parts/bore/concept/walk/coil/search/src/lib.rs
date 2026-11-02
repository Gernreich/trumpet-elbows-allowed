//! The coil search's tools, shared code. Each program in tools/ is a thin front end
//! over these modules.
//!
//! The tools were JavaScript until 2026-11-02. README.md, SCORING.md and the .json
//! files were written by Node, and the gate regenerates them and requires them to
//! come out byte-identical, so `jsfmt` writes numbers by JavaScript's rules and
//! `json` lays out JSON as JSON.stringify did.

pub mod bore;
pub mod corpus;
pub mod iterate;
pub mod jsfmt;
pub mod json;
pub mod minimal;
pub mod reduce;
pub mod score;
pub mod spiral_metrics;
pub mod standardise;
pub mod table;

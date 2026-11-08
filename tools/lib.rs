//! The trumpet tools' shared code. snakebox.rs is GPL-3.0-or-later (it is
//! Boxes.py's drawing in Rust); the rest of this crate is CC0, and a program
//! built with snakebox.rs in it is distributed under the GPL.
pub mod assemble;
pub mod bore;
pub mod designs;
pub mod check;
pub mod geom;
pub mod snakebox;
pub mod viewer;
pub mod pyrandom;
pub mod render;
pub mod viewer_page;
pub mod xml;
/// The parts tools' reading of a ring sheet, which the viewer uses to draw the
/// bell and mouthpiece on every page. Shared with parts/, not copied.
#[path = "../parts/common.rs"]
pub mod common;
#[path = "../parts/views.rs"]
pub mod views;

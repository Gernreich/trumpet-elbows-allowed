//! `bore_render WALK [--directions] [--out=FILE]`: six still views of a bore,
//! coloured by piece or by direction of travel. bore_render.py until
//! 2026-11-08. Writes bore3d.svg (bore3d_directions.svg) in the working
//! directory unless --out names a file.
use trumpet::bore::{sys_exit, Design};

fn main() {
    let mut argv: Vec<String> = std::env::args().skip(1).collect();
    let mut by_dir = false;
    for flag in ["--directions", "--by-direction"] {
        if let Some(i) = argv.iter().position(|a| a == flag) {
            argv.remove(i);
            by_dir = true;
        }
    }
    let mut dest = None;
    for a in argv.clone() {
        if let Some(v) = a.strip_prefix("--out=") {
            argv.retain(|x| x != &a);
            dest = Some(v.to_string());
        }
    }
    if let Some(b) = argv.iter().find(|a| a.starts_with('-')) {
        sys_exit(&format!("unknown option {b}: --directions or --out=FILE"));
    }
    let mut o = String::new();
    let r = Design::default().render_main(&argv.join(" "), by_dir, dest.as_deref(), &mut o);
    print!("{o}");
    if let Err(e) = r {
        println!("error: {e}");
        std::process::exit(1);
    }
}

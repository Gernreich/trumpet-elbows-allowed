use trumpet::bore::{walk_text, Design};

fn main() {
    let mut d = Design::default();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut a = d.take_design_switches(&argv);
    if let Some(i) = a.iter().position(|x| x.starts_with("--title=")) {
        d.title = Some(a.remove(i).split_once('=').unwrap().1.to_string());
    }
    let mut dest: Option<String> = Some(format!("{}/tools/../../test", trumpet::common::repo()));
    if let Some(i) = a.iter().position(|x| x == "--no-write") {
        a.remove(i);
        dest = None;
    }
    if let Some(i) = a.iter().position(|x| x == "--write") {
        dest = Some(a[i + 1].clone());
        a.drain(i..i + 2);
    }
    let mut o = String::new();
    let r = (if a.is_empty() { Ok("D R1 F".to_string()) } else { walk_text(&a.join(" ")) }).and_then(|t| d.main(&t, dest.as_deref(), &mut o));
    if let Err(e) = r {
        o.push_str(&format!("error: {e}\n"));
        print!("{o}");
        std::process::exit(1);
    }
    print!("{o}");
}

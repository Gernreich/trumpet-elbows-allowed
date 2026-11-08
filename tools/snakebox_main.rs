//! `snakebox [SnakeBoxVar options]`: draw one piece and list its paths, one
//! group per line (`G p-0`) and one path per line (`P rgb(0,0,0) M ...`). It
//! takes the options Boxes.py's `scripts/boxes SnakeBoxVar` took. A refusal
//! is printed as `ValueError: ...` on stderr, exit 1, as Boxes.py printed it.
use trumpet::snakebox::Snake;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = Snake::from_args(&args).and_then(|s| s.render());
    match r {
        Ok(groups) => {
            let mut out = String::new();
            for (id, paths) in groups {
                out += &format!("G {id}\n");
                for (d, c) in paths {
                    out += &format!("P {c} {d}\n");
                }
            }
            print!("{out}");
        }
        Err(e) => {
            eprintln!("ValueError: {e}");
            std::process::exit(1);
        }
    }
}

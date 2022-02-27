use core::mem;
use core::str::FromStr;
use std::env;
use std::ffi;
use std::fs;
use std::io;
use std::io::BufRead;
use std::process;

struct Args {
    file: Option<ffi::OsString>,
    lines: bool,
}

fn parse_args() -> Args {
    let mut args = env::args_os().collect::<Vec<_>>();

    let usage = || {
        let program_name = args
            .get(0)
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from_str("lynx-sudoku").unwrap());
        _ = eprintln!("Usage: {} [--lines] [FILE]", program_name);
        process::exit(1);
    };

    match args.len() {
        0 | 1 => Args {
            file: None,
            lines: false,
        },
        2 => {
            if args[1].to_str() == Some("--lines") {
                Args {
                    file: None,
                    lines: true,
                }
            } else {
                Args {
                    file: Some(mem::take(&mut args[1])),
                    lines: false,
                }
            }
        }
        3 => {
            if args[1].to_str() != Some("--lines") {
                usage();
            }
            Args {
                file: Some(mem::take(&mut args[2])),
                lines: true,
            }
        }
        _ => usage(),
    }
}

fn main() {
    let args = parse_args();

    let mut file: Box<dyn io::BufRead> = match args.file {
        Some(file) => Box::new(io::BufReader::new(fs::File::open(file).unwrap())), // FIXME
        None => Box::new(io::BufReader::new(io::stdin())),
    };

    if args.lines {
        for line in file.lines() {
            let sudoku = lynx::Sudoku::from_str(&line.unwrap()).unwrap(); // FIXME
            let solved = sudoku.solve();
            println!("{}", solved.unwrap_or(sudoku).to_string_line());
        }
    } else {
        let string = {
            let mut string = String::new();
            file.read_to_string(&mut string).unwrap(); // FIXME
            string
        };
        let sudoku = lynx::Sudoku::from_str(&string).unwrap(); // FIXME
        let solved = sudoku.solve();
        println!("{}", solved.unwrap_or(sudoku));
    }
}

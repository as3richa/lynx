use core::fmt;
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
        _ = eprintln!(
            "Usage: {} [--lines] [FILE]\n\
             \n\
             Solve one or several Sudoku puzzles given in FILE. FILE defaults to the\n\
             standard input. If --lines is specified, each line of FILE is parsed and\n\
             solved as a separate puzzle, otherwise the whole file parsed as one single\n\
             puzzle.\n\
             \n\
             Puzzles are specified using the digits 1 through 9 to represent occupied\n\
             cells, and '.' or 0 to represent unoccupied cells. Whitespace is ignored,\n\
             except for newlines in --lines mode. Puzzles with no solution are echoed\n\
             verbatim.",
            program_name
        );
        process::exit(1);
    };

    match args.len() {
        0 | 1 => Args {
            file: None,
            lines: false,
        },
        2 => match args[1].to_str() {
            Some("--help" | "-h") => usage(),
            Some("--lines") => Args {
                file: None,
                lines: true,
            },
            _ => Args {
                file: Some(mem::take(&mut args[1])),
                lines: false,
            },
        },
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
        Some(file) => Box::new(io::BufReader::new(
            fs::File::open(file).unwrap_or_else(handle_error),
        )),
        None => Box::new(io::BufReader::new(io::stdin())),
    };

    if args.lines {
        for line in file.lines() {
            let sudoku = lynx::sudoku::Sudoku::from_str(&line.unwrap_or_else(handle_error))
                .unwrap_or_else(handle_error);
            let solved = sudoku.solve();
            println!("{}", solved.unwrap_or(sudoku).to_string_line());
        }
    } else {
        let string = {
            let mut string = String::new();
            file.read_to_string(&mut string)
                .unwrap_or_else(handle_error);
            string
        };
        let sudoku = lynx::sudoku::Sudoku::from_str(&string).unwrap_or_else(handle_error);
        let solved = sudoku.solve();
        println!("{}", solved.unwrap_or(sudoku));
    }
}

fn handle_error<E: fmt::Display, R>(error: E) -> R {
    _ = eprintln!("{}", error);
    process::exit(1);
}

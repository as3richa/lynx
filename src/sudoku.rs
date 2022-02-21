use crate::DLXMatrix;

use core::fmt;
use core::str;

#[derive(Default, Debug)]
pub struct Sudoku {
    grid: [[u8; 9]; 9],
}

impl str::FromStr for Sudoku {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, ()> {
        let mut sudoku = Self::new();

        let mut i = 0;

        for ch in string.chars() {
            match ch {
                '0' | '.' | ' ' | '_' => i += 1,
                '1'..='9' => {
                    sudoku.set(i % 9, i / 9, (ch as u8) - b'0');
                    i += 1;
                }
                _ => (),
            }

            if i == 81 {
                break;
            }
        }

        Ok(sudoku)
    }
}

impl Sudoku {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, x: usize, y: usize, value: u8) {
        assert!(value <= 9);
        self.grid[x][y] = value;
    }

    pub fn clear(&mut self, x: usize, y: usize) {
        self.grid[x][y] = 0;
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.grid[x][y]
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, u8)> + '_ {
        (0..9).flat_map(move |x| {
            let x = x;
            (0..9).map(move |y| (x, y, self.grid[x][y]))
        })
    }

    fn constraints(&self) -> Result<SudokuConstraints, ()> {
        let mut constraints = SudokuConstraints::new();

        for (x, y, value) in self.iter().filter(|&(_, _, value)| value != 0) {
            constraints.add(x, y, value)?;
        }

        Ok(constraints)
    }

    pub fn solve(&self) -> Option<Sudoku> {
        let constraints = self.constraints().ok()?;

        let mut matrix = DLXMatrix::<u16>::new(324);

        let mut push_row = |x: usize, y: usize, value: u8| {
            let value = (value - 1) as u16;
            let box_id = SudokuConstraints::box_id(x, y);
            matrix.push_row(&[
                9 * (y as u16) + (x as u16),
                81 + 9 * (y as u16) + value,
                162 + 9 * (x as u16) + value,
                243 + 9 * (box_id as u16) + value,
            ]);
        };

        for (x, y, value) in self.iter() {
            if value == 0 {
                for value in constraints.get_candidates(x, y) {
                    push_row(x, y, value);
                }
            } else {
                push_row(x, y, value);
            }
        }

        let mut solution = matrix.solve()?;

        let mut solved = Sudoku::new();

        // FIXME: ???
        while let Some(mut row) = solution.next() {
            let mut elements = vec![];
            while let Some(element) = row.next(&solution) {
                elements.push(element);
            }
            elements.sort_unstable();

            let x = (elements[0] % 9) as usize;
            let y = (elements[0] / 9) as usize;
            let value = (elements[1] % 9 + 1) as u8;
            solved.set(x, y, value);
        }

        Some(solved)
    }
}

impl fmt::Display for Sudoku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        for y in 0..9 {
            for x in 0..9 {
                let ch = if self.grid[x][y] == 0 {
                    b'.'
                } else {
                    self.grid[x][y] + b'0'
                };
                write!(f, "{}", ch as char)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

struct SudokuConstraints {
    rows: [u16; 9],
    columns: [u16; 9],
    boxes: [u16; 9],
}

impl SudokuConstraints {
    fn new() -> Self {
        Self {
            rows: [((1 << 9) - 1); 9],
            columns: [((1 << 9) - 1); 9],
            boxes: [((1 << 9) - 1); 9],
        }
    }

    fn add(&mut self, x: usize, y: usize, value: u8) -> Result<(), ()> {
        debug_assert!((1..=9).contains(&value));
        let value = value - 1;

        let flags_refs = [
            &mut self.rows[y],
            &mut self.columns[x],
            &mut self.boxes[Self::box_id(x, y)],
        ];

        for flags in flags_refs {
            if *flags & (1 << value) == 0 {
                return Err(());
            }
            *flags ^= 1 << value;
        }

        Ok(())
    }

    fn get_candidates(&self, x: usize, y: usize) -> impl Iterator<Item = u8> + '_ {
        debug_assert!((0..9).contains(&x) && (0..9).contains(&y));
        CandidateIterator(self.rows[y] & self.columns[x] & self.boxes[Self::box_id(x, y)])
    }

    fn box_id(x: usize, y: usize) -> usize {
        3 * (y / 3) + (x / 3)
    }
}

struct CandidateIterator(u16);

impl Iterator for CandidateIterator {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.0 == 0 {
            return None;
        }

        let value = (self.0.trailing_zeros() + 1) as u8;
        self.0 ^= 1 << (value - 1);

        Some(value)
    }
}

#[cfg(test)]
mod test {
    use crate::Sudoku;
    use core::str::FromStr;
    use std::fs;
    use std::io;
    use std::io::BufRead;

    fn validate_solution(sudoku: Sudoku) {
        let solution = sudoku.solve().unwrap();

        assert!(solution.constraints().is_ok());

        for (x, y, value) in sudoku.iter() {
            if value == 0 {
                continue;
            }
            assert!(solution.get(x, y) == value);
        }
    }

    #[test]
    fn test_empty() {
        let sudoku = Sudoku::new();
        validate_solution(sudoku);
    }

    #[test]
    fn test_top1465() {
        let file = fs::File::open("data/sudoku/top1465.list").unwrap();
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            let sudoku = Sudoku::from_str(trimmed).unwrap();
            validate_solution(sudoku);
        }
    }
}

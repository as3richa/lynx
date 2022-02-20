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
                '0' | '.' | ' ' => i += 1,
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
        assert!(x < 9 && y < 9);
        self.grid[x][y] = 0;
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.grid[x][y]
    }

    pub fn solve(&mut self) -> bool {
        let mut rows = [0u16; 9];
        let mut columns = [0u16; 9];
        let mut blocks = [0u16; 9];

        for x in 0..9 {
            for y in 0..9 {
                if self.grid[x][y] == 0 {
                    continue;
                }

                let value = self.grid[x][y] - 1;

                let block = 3 * (y / 3) + x / 3;

                let flags_refs = [&mut rows[y], &mut columns[x], &mut blocks[block]];

                for flags in flags_refs {
                    if (*flags >> value) & 1 != 0 {
                        return false;
                    }
                    *flags |= 1 << value
                }
            }
        }

        let mut matrix = DLXMatrix::<u16>::new(324);

        for x in 0..9 {
            for y in 0..9 {
                let block = 3 * (y / 3) + x / 3;

                if self.grid[x][y] != 0 {
                    let value = (self.grid[x][y] - 1) as u16;

                    matrix.push_row(&[
                        9 * (y as u16) + (x as u16),
                        81 + 9 * (y as u16) + value,
                        162 + 9 * (x as u16) + value,
                        243 + 9 * (block as u16) + value,
                    ]);
                } else {
                    let flags = rows[y] | columns[x] | blocks[block];

                    for value in 0..9 {
                        if (flags >> value) & 1 != 0 {
                            continue;
                        }

                        matrix.push_row(&[
                            9 * (y as u16) + (x as u16),
                            81 + 9 * (y as u16) + value,
                            162 + 9 * (x as u16) + value,
                            243 + 9 * (block as u16) + value,
                        ]);
                    }
                }
            }
        }

        match matrix.solve() {
            Some(mut solution) => {
                while let Some(mut row) = solution.next() {
                    let mut elements = vec![];
                    while let Some(element) = row.next(&solution) {
                        elements.push(element);
                    }
                    elements.sort_unstable();

                    let x = (elements[0] % 9) as usize;
                    let y = (elements[0] / 9) as usize;
                    let value = (elements[1] % 9 + 1) as u8;
                    self.grid[x][y] = value;
                }
                true
            }
            None => false,
        }
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

#[cfg(test)]
mod test {
    use crate::Sudoku;
    use core::str::FromStr;

    #[test]
    fn test_simple() {
        let mut game = Sudoku::from_str(
            "4...3.......6..8..........1....5..9..8....6...7.2........1.27..5.3....4.9........",
        )
        .unwrap();
        println!("{}", game);
        println!("{}", game.solve());
        println!("{}", game);
    }
}

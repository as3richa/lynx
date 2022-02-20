use core::cmp;
use core::fmt;
use std::vec;

mod sudoku;
pub use sudoku::Sudoku;

fn on_integer_overflow<T>() -> T {
    panic!("Integer overflow");
}

pub trait Size:
    num::Unsigned + num::ToPrimitive + num::FromPrimitive + Copy + Ord + fmt::Display + fmt::Debug
{
    fn to_usize_unwrap(self) -> usize {
        self.to_usize().unwrap_or_else(on_integer_overflow)
    }

    fn from_usize_unwrap(value: usize) -> Self {
        Self::from_usize(value).unwrap_or_else(on_integer_overflow)
    }
}

impl Size for u8 {}
impl Size for u16 {}
impl Size for u32 {}
impl Size for u64 {}
impl Size for usize {}

struct Node<S: Size> {
    left: S,
    right: S,
    up: S,
    down: S,
    column: S,
}

pub struct DLXMatrix<S: Size> {
    columns: S,
    buffer: Vec<Node<S>>,
}

impl<S: Size> DLXMatrix<S> {
    pub fn new(columns: S) -> Self {
        let buffer = (0..=columns.to_usize_unwrap())
            .map(|i| {
                let i = S::from_usize_unwrap(i);

                let left = if i.is_zero() { columns } else { i - S::one() };

                let right = if i == columns {
                    S::zero()
                } else {
                    i + S::one()
                };

                Node {
                    left,
                    right,
                    up: i,
                    down: i,
                    column: S::zero(),
                }
            })
            .collect();
        DLXMatrix { columns, buffer }
    }

    pub fn columns(&self) -> S {
        self.columns
    }

    pub fn push_row(&mut self, columns: &[S]) {
        assert!(!columns.is_empty(), "Rows must be non-empty");

        let row = self.buffer.len();
        self.buffer.reserve(columns.len());

        for (i, &column) in columns.iter().enumerate() {
            assert!(
                column < self.columns,
                "Columns must be in the range 0..{} (got {})",
                self.columns,
                column
            );

            let node = S::from_usize_unwrap(row + i);
            let left = S::from_usize_unwrap(row + if i == 0 { columns.len() - 1 } else { i - 1 });
            let right = S::from_usize_unwrap(row + if i == columns.len() - 1 { 0 } else { i + 1 });

            let up = {
                let column_ref = unsafe { self.get_unchecked_mut(column) };
                let up = column_ref.up;
                column_ref.up = node;
                column_ref.column = column_ref.column + S::one();
                up
            };

            let up_ref = unsafe { self.get_unchecked_mut(up) };
            up_ref.down = node;

            let down = column;

            let node_val = Node {
                left,
                right,
                up,
                down,
                column,
            };

            {
                let len = S::from_usize_unwrap(self.buffer.len() + columns.len());
                debug_assert!(node < len && left < len && right < len && up < len && column < len);

                unsafe {
                    debug_assert!(
                        self.get_unchecked(up).down == node && self.get_unchecked(down).up == node
                    );
                }
            }

            unsafe {
                self.buffer
                    .as_mut_ptr()
                    .add(node.to_usize_unwrap())
                    .write(node_val);
            }
        }

        for i in 0..columns.len() {
            let node = S::from_usize_unwrap(row + i);
            unsafe {
                let node_ref = self.get_unchecked(node);
                debug_assert!(
                    self.get_unchecked(node_ref.left).right == node
                        && self.get_unchecked(node_ref.right).left == node
                );
            }
        }

        unsafe {
            self.buffer.set_len(self.buffer.len() + columns.len());
        }
    }

    pub fn solve(mut self) -> Option<Solution<S>> {
        let mut rows = vec![];
        if self.solve_recursive(&mut rows) {
            Some(Solution {
                matrix: self,
                rows: rows.into_iter(),
            })
        } else {
            None
        }
    }

    unsafe fn get_unchecked(&self, i: S) -> &Node<S> {
        self.buffer.get_unchecked(S::to_usize_unwrap(i))
    }

    unsafe fn get_unchecked_mut(&mut self, i: S) -> &mut Node<S> {
        self.buffer.get_unchecked_mut(S::to_usize_unwrap(i))
    }

    fn solve_recursive(&mut self, solution: &mut Vec<S>) -> bool {
        //println!("Depth: {}", solution.len());
        if let Some(column) = self.choose_column() {
            let mut rows = ColumnIterator::new(column);
            rows.next(self);

            while let Some(row) = rows.next(self) {
                unsafe {
                    self.select_row(row);
                }
                solution.push(row);

                if self.solve_recursive(solution) {
                    return true;
                }

                unsafe {
                    self.deselect_row(row);
                }
                solution.pop();
            }

            false
        } else {
            true
        }
    }

    fn choose_column(&self) -> Option<S> {
        {
            //println!("Columns:");

            let mut columns = RowIterator::new(self.columns);
            columns.next(self);

            let mut columns_vec = vec![];
            while let Some(column) = columns.next(self) {
                columns_vec.push(column);
            }

            println!("{:?}", columns_vec.len());
        }
        let mut columns = RowIterator::new(self.columns);
        columns.next(self);

        if let Some(first_column) = columns.next(self) {
            let mut best = unsafe { (self.get_unchecked(first_column).column, first_column) };

            while let Some(column) = columns.next(self) {
                best = cmp::min(best, unsafe { (self.get_unchecked(column).column, column) });
            }

            Some(best.1)
        } else {
            None
        }
    }

    unsafe fn select_row(&mut self, row: S) {
        let mut elements = RowIterator::new(row);

        while let Some(element) = elements.next(self) {
            let column = self.get_unchecked(element).column;

            let mut conflicting_rows = ColumnIterator::new(column);
            conflicting_rows.next(self);

            while let Some(row) = conflicting_rows.next(self) {
                self.remove_row(row);
            }

            self.remove_column(column);
        }
    }

    unsafe fn deselect_row(&mut self, row: S) {
        let mut elements = ReverseRowIterator::new(self.get_unchecked(row).left);

        while let Some(element) = elements.next(self) {
            let column = self.get_unchecked(element).column;

            self.restore_column(column);

            let mut conflicting_rows = ReverseColumnIterator::new(column);
            conflicting_rows.next(self);

            while let Some(row) = conflicting_rows.next(self) {
                self.restore_row(row);
            }
        }
    }

    unsafe fn remove_row(&mut self, row: S) {
        let mut elements = RowIterator::new(row);
        elements.next(self);

        //println!("Row removed: {}", row);

        while let Some(element) = elements.next(self) {
            let (column, up, down) = {
                let element_ref = self.get_unchecked_mut(element);
                (element_ref.column, element_ref.up, element_ref.down)
            };

            self.get_unchecked_mut(column).column =
                self.get_unchecked_mut(column).column - S::one();

            debug_assert!(self.get_unchecked(up).down == element);
            self.get_unchecked_mut(up).down = down;

            debug_assert!(self.get_unchecked(down).up == element);
            self.get_unchecked_mut(down).up = up;
        }
    }

    unsafe fn restore_row(&mut self, row: S) {
        let mut elements = RowIterator::new(row);
        elements.next(self);

        //println!("Row restored: {}", row);

        while let Some(element) = elements.next(self) {
            let (column, up, down) = {
                let element_ref = self.get_unchecked_mut(element);
                (element_ref.column, element_ref.up, element_ref.down)
            };

            self.get_unchecked_mut(column).column =
                self.get_unchecked_mut(column).column + S::one();

            debug_assert!(self.get_unchecked(up).down == down);
            self.get_unchecked_mut(up).down = element;

            debug_assert!(self.get_unchecked(down).up == up);
            self.get_unchecked_mut(down).up = element;
        }
    }

    unsafe fn remove_column(&mut self, column: S) {
        debug_assert!(column < self.columns);

        let (left, right) = {
            let column_ref = self.get_unchecked_mut(column);
            (column_ref.left, column_ref.right)
        };

        debug_assert!(self.get_unchecked(left).right == column);
        self.get_unchecked_mut(left).right = right;

        debug_assert!(self.get_unchecked(right).left == column);
        self.get_unchecked_mut(right).left = left;
    }

    unsafe fn restore_column(&mut self, column: S) {
        let (left, right) = {
            let column_ref = self.get_unchecked_mut(column);
            (column_ref.left, column_ref.right)
        };

        debug_assert!(self.get_unchecked(left).right == right);
        self.get_unchecked_mut(left).right = column;

        debug_assert!(self.get_unchecked(right).left == left);
        self.get_unchecked_mut(right).left = column;
    }
}

macro_rules! dlx_matrix_iter_impl {
    ($name:ident, $next:ident) => {
        struct $name<S: Size> {
            row: S,
            cursor: S,
            exhausted: bool,
        }

        impl<S: Size> $name<S> {
            fn new(row: S) -> Self {
                Self {
                    row,
                    cursor: row,
                    exhausted: false,
                }
            }

            fn next(&mut self, matrix: &DLXMatrix<S>) -> Option<S> {
                if self.exhausted {
                    return None;
                }

                let item = self.cursor;

                let next = unsafe { matrix.get_unchecked(self.cursor).$next };

                if next == self.row {
                    self.exhausted = true;
                } else {
                    self.cursor = next;
                }

                Some(item)
            }
        }
    };
}

dlx_matrix_iter_impl!(RowIterator, right);
dlx_matrix_iter_impl!(ReverseRowIterator, left);
dlx_matrix_iter_impl!(ColumnIterator, down);
dlx_matrix_iter_impl!(ReverseColumnIterator, up);

pub struct Solution<S: Size> {
    matrix: DLXMatrix<S>,
    rows: vec::IntoIter<S>,
}

impl<S: Size> Iterator for Solution<S> {
    type Item = SolutionRow<S>;

    fn next(&mut self) -> Option<SolutionRow<S>> {
        self.rows.next().map(SolutionRow::new)
    }
}

pub struct SolutionRow<S: Size> {
    row: S,
    cursor: S,
    exhausted: bool,
}

impl<S: Size> SolutionRow<S> {
    fn new(row: S) -> Self {
        Self {
            row,
            cursor: row,
            exhausted: false,
        }
    }

    pub fn next(&mut self, solution: &Solution<S>) -> Option<S> {
        if self.exhausted {
            return None;
        }

        let (column, right) = unsafe {
            let cursor_ref = solution.matrix.get_unchecked(self.cursor);
            (cursor_ref.column, cursor_ref.right)
        };

        if right == self.row {
            self.exhausted = true;
        } else {
            self.cursor = right;
        }

        Some(column)
    }
}

#[cfg(test)]
mod test {
    use crate::DLXMatrix;

    #[test]
    fn test_simple() {
        let mut matrix = DLXMatrix::new(5usize);
        matrix.push_row(&[0]);
        matrix.push_row(&[1]);
        matrix.push_row(&[2]);
        matrix.push_row(&[3]);
        matrix.push_row(&[4]);

        let mut solution = matrix.solve().unwrap();

        while let Some(mut row) = solution.next() {
            while let Some(column) = row.next(&solution) {}
        }
    }

    #[test]
    fn test_simple2() {
        let mut matrix = DLXMatrix::new(5usize);
        matrix.push_row(&[0, 1, 2]);
        matrix.push_row(&[1, 2, 3]);
        matrix.push_row(&[1, 2, 3, 4]);
        matrix.push_row(&[0, 4]);
        matrix.push_row(&[0, 1]);
        matrix.push_row(&[2, 3]);

        let mut solution = matrix.solve().unwrap();
        //println!("Done");

        while let Some(mut row) = solution.next() {
            let mut row_vec = vec![];
            while let Some(column) = row.next(&solution) {
                row_vec.push(column);
            }

            //println!("Solution row: {:?}", row_vec);
        }
    }
}

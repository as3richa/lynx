use core::cmp::min;
use core::hash::Hash;
use num::{range, FromPrimitive, ToPrimitive, Unsigned};
use std::collections::HashMap;

fn on_integer_overflow<T>() -> T {
    panic!("Integer overflow");
}

pub trait Size: Unsigned + ToPrimitive + FromPrimitive + Copy + Ord + Hash {
    fn to_usize_unwrap(self) -> usize {
        ToPrimitive::to_usize(&self).unwrap_or_else(on_integer_overflow)
    }

    fn from_usize_unwrap(value: usize) -> Self {
        FromPrimitive::from_usize(value).unwrap_or_else(on_integer_overflow)
    }
}

struct Node<T: Size> {
    left: T,
    right: T,
    up: T,
    down: T,
    data: T,
}

pub struct DLXMatrix<T: Size, X> {
    columns: T,
    column_header: T,
    buffer: Vec<Node<T>>,
    row_data: HashMap<T, X>,
}

impl<T: Size, X> DLXMatrix<T, X> {
    pub fn new(columns: T) -> Self {
        let buffer = range(T::zero(), columns)
            .map(|column| {
                let left = if column.is_zero() {
                    columns - T::one()
                } else {
                    column - T::one()
                };

                let right = if column == columns - T::one() {
                    T::zero()
                } else {
                    column + T::one()
                };

                Node {
                    left,
                    right,
                    up: column,
                    down: column,
                    data: T::zero(),
                }
            })
            .collect::<Vec<_>>();

        Self {
            buffer,
            columns,
            column_header: T::zero(),
            row_data: HashMap::new(),
        }
    }

    pub fn append_row(&mut self, columns: &[T], data: X) {
        assert!(!columns.is_empty());

        self.buffer.reserve(columns.len());

        let row = T::from_usize_unwrap(self.buffer.len());

        for (i, &column) in columns.iter().enumerate() {
            assert!(column < self.columns);

            let node = row + T::from_usize_unwrap(i);

            let left = if i == 0 {
                row + T::from_usize_unwrap(columns.len()) - T::one()
            } else {
                node - T::one()
            };

            let right = if i == columns.len() - 1 {
                row
            } else {
                node + T::one()
            };

            let down = column;

            let up = self.node(column).up;
            self.node_mut(up).down = node;

            self.buffer.push(Node {
                left,
                right,
                up,
                down,
                data: row,
            });

            self.node_mut(column).data = self.node(column).data + T::one();
        }

        self.row_data.insert(row, data);
    }

    fn node(&self, node: T) -> &Node<T> {
        &self.buffer[node.to_usize_unwrap()]
    }

    fn node_mut(&mut self, node: T) -> &mut Node<T> {
        &mut self.buffer[node.to_usize_unwrap()]
    }

    unsafe fn node_unchecked(&self, node: T) -> &Node<T> {
        self.buffer.get_unchecked(node.to_usize_unwrap())
    }

    unsafe fn node_unchecked_mut(&mut self, node: T) -> &mut Node<T> {
        self.buffer.get_unchecked_mut(node.to_usize_unwrap())
    }

    fn node_is_column(&self, node: T) -> bool {
        node < self.columns
    }

    fn min_count_column(&self) -> T {
        let mut column_iter = RowIterator::new(self.column_header);
        let mut min_count_column = None;

        while let Some(column) = unsafe { column_iter.next(&self) } {
            min_count_column = {
                let count = unsafe { self.node_unchecked(column) }.data;

                Some(match min_count_column {
                    Some(best) => min(best, (column, count)),
                    None => (column, count),
                })
            }
        }

        min_count_column.unwrap().1
    }

    unsafe fn remove_row(&mut self, node: T) {
        unimplemented!();
    }

    unsafe fn restore_row(&mut self, node: T) {
        unimplemented!();
    }

    unsafe fn remove_column(&mut self, column: T) {
        unimplemented!();
    }

    unsafe fn restore_column(&mut self, column: T) {
        unimplemented!();
    }
}

struct Solver<T: Size, X> {
    matrix: DLXMatrix<T, X>,
    stack: Vec<T>,
    header: Option<T>,
}

impl<T: Size, X> Solver<T, X> {
    fn solve(&mut self) -> Option<&Vec<T>> {
        let header = match self.header {
            Some(header) => header,
            None => return Some(&self.stack),
        };

        let column = self.matrix.min_count_column();

        let mut column_iter = ColumnIterator::new(column);
        unsafe { column_iter.next(&self.matrix) };

        while let Some(node) = unsafe { column_iter.next(&self.matrix) } {
            if self.matrix.node_is_column(node) {
                continue;
            }

            let mut row_iter = RowIterator::new(node);

            while let Some(node) = unsafe { row_iter.next(&self.matrix) } {
                let mut column_iter = ColumnIterator::new(node);

                while let Some(node) = unsafe { column_iter.next(&self.matrix) } {
                    if self.matrix.node_is_column(node) {
                        unsafe {
                            self.matrix.remove_column(node);
                        }
                    } else {
                        unsafe {
                            self.matrix.remove_row(node);
                        }
                    }
                }
            }

            // constrain wrt node
            self.stack.push(node);
            self.solve();
            // unconstrain
        }

        unimplemented!();
    }
}

macro_rules! matrix_iterator_impl {
    ($id:ident, $right:ident) => {
        struct $id<T: Size> {
            node: T,
            start: T,
            exhausted: bool,
        }

        impl<T: Size> $id<T> {
            fn new(node: T) -> Self {
                Self {
                    node,
                    start: node,
                    exhausted: false,
                }
            }

            unsafe fn next<X>(&mut self, matrix: &DLXMatrix<T, X>) -> Option<T> {
                if self.exhausted {
                    return None;
                }

                let node = self.node;
                self.node = matrix.node_unchecked(self.node).$right;

                if self.node == self.start {
                    self.exhausted = true;
                }

                Some(node)
            }
        }
    };
}

matrix_iterator_impl!(RowIterator, right);
matrix_iterator_impl!(ColumnIterator, down);

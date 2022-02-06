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
}

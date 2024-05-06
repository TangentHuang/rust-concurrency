use crate::{dot_product, Vector};
use anyhow::Result;
use std::fmt::Formatter;
use std::ops::{Add, AddAssign, Mul};
use std::sync::mpsc;
use std::{fmt, thread};

const NUM_THREADS: usize = 4;
pub struct Matrix<T> {
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

pub struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}
pub struct MsgOutput<T> {
    idx: usize,
    value: T,
}

pub struct Msg<T> {
    input: MsgInput<T>,
    sender: oneshot::Sender<MsgOutput<T>>,
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Copy + Default + Mul<Output = T> + Add<Output = T> + AddAssign + Send + 'static,
{
    if a.cols != b.rows {
        return Err(anyhow::anyhow!("Invalid matrix dimensions"));
    }

    let senders = (0..NUM_THREADS)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<Msg<T>>();
            thread::spawn(move || {
                for msg in rx {
                    let value = dot_product(msg.input.row, msg.input.col)?;
                    if let Err(e) = msg.sender.send(MsgOutput {
                        idx: msg.input.idx,
                        value,
                    }) {
                        eprintln!("Error sending message: {:?}", e);
                    };
                }
                Ok::<_, anyhow::Error>(())
            });
            tx
        })
        .collect::<Vec<_>>();

    let rows = a.rows;
    let cols = b.cols;
    let mut result = Matrix::new(rows, cols, vec![T::default(); rows * cols]);
    let mut receivers = Vec::with_capacity(rows * cols);
    for i in 0..rows {
        for j in 0..cols {
            let row = Vector::new(&a.data[i * a.cols..(i + 1) * a.cols]);
            let col_data = b.data[j..]
                .iter()
                .step_by(b.cols)
                .copied()
                .collect::<Vec<_>>();
            let col = Vector::new(col_data);
            let idx = i * cols + j;
            let input = MsgInput::new(idx, row, col);
            let (tx, rx) = oneshot::channel();
            let msg = Msg::new(input, tx);
            if let Err(e) = senders[idx % NUM_THREADS].send(msg) {
                eprintln!("Error sending message: {:?}", e);
            }
            receivers.push(rx);
        }
    }
    for rx in receivers {
        let output = rx.recv()?;
        result.data[output.idx] = output.value;
    }

    Ok(result)
}

impl<T> Matrix<T> {
    pub fn new(rows: usize, cols: usize, data: impl Into<Vec<T>>) -> Self {
        Self {
            data: data.into(),
            rows,
            cols,
        }
    }
}

impl<T> Mul for Matrix<T>
where
    T: Copy + Default + Mul<Output = T> + Add<Output = T> + Send + std::ops::AddAssign + 'static,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        multiply(&self, &rhs).unwrap()
    }
}

impl<T> fmt::Display for Matrix<T>
where
    T: fmt::Display,
{
    //display a  2x3 as {1 2 3, 4 5 6}, 3x2 as {1 4, 2 5, 3 6}
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for i in 0..self.rows {
            for j in 0..self.cols {
                write!(f, "{}", self.data[i * self.cols + j])?;
                if j != self.cols - 1 {
                    write!(f, " ")?;
                }
            }
            if i != self.rows - 1 {
                write!(f, ",")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<T> fmt::Debug for Matrix<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Matrix(row={},col={},{})", self.rows, self.cols, self)
    }
}

impl<T> MsgInput<T> {
    fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}

impl<T> Msg<T> {
    fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_matrix() -> Result<()> {
        let a = Matrix::new(2, 3, vec![1, 2, 3, 4, 5, 6]);
        let b = Matrix::new(3, 2, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(format!("{:?}", a), "Matrix(row=2,col=3,{1 2 3,4 5 6})");
        assert_eq!(format!("{:?}", b), "Matrix(row=3,col=2,{1 2,3 4,5 6})");

        let c = a * b;
        assert_eq!(format!("{:?}", c), "Matrix(row=2,col=2,{22 28,49 64})");

        Ok(())
    }

    #[test]
    fn test_matrix_mul_error() {
        let a = Matrix::new(2, 3, vec![1, 2, 3, 4, 5, 6]);
        let b = Matrix::new(2, 3, vec![1, 2, 3, 4, 5, 6]);
        let c = multiply(&a, &b);
        assert!(c.is_err());
    }
    #[test]
    #[should_panic]
    fn test_matrix_panic() {
        let a = Matrix::new(2, 3, vec![1, 2, 3, 4, 5, 6]);
        let b = Matrix::new(2, 3, vec![1, 2, 3, 4, 5, 6]);
        _ = a * b
    }
}

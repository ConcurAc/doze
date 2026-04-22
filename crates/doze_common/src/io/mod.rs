mod slice;

pub use slice::{SliceReader, SliceWriter};

use crate::sample::Sample;

pub trait Reader<T> {
    /// Read samples
    fn read(&mut self, output: &mut [T]) -> usize;
}

pub trait ReaderExt<T>: Reader<T> {
    /// Restart reading
    fn restart_read(&mut self);
    /// Get the remaining samples for reading
    fn remaining_read(&self) -> usize;
    /// is finished reading
    fn is_read_finished(&self) -> bool {
        self.remaining_read() == 0
    }
}

pub trait Writer<T> {
    /// Write samples
    fn write(&mut self, input: &[T]) -> usize;
}

pub trait WriterExt<T>: Writer<T> {
    /// Restart writing
    fn restart_write(&mut self);
    /// Get the remaining samples for writing
    fn remaining_write(&self) -> usize;
    /// is finished writing
    fn is_write_finished(&self) -> bool {
        self.remaining_write() == 0
    }
}

pub fn apply<T: Sample, const N: usize>(
    mut input: impl Reader<T>,
    mut output: impl Writer<T>,
    mut apply: impl FnMut(T) -> T,
) {
    let mut chunk = [T::zeroed(); N];
    loop {
        let n = input.read(&mut chunk);
        for i in 0..n {
            chunk[i] = apply(chunk[i]);
        }
        output.write(&chunk[..n]);
        if n < N {
            break;
        }
    }
}

pub fn apply_chunks<T: Sample, const N: usize>(
    mut input: impl Reader<T>,
    mut output: impl Writer<T>,
    mut apply: impl FnMut(&mut [T]),
) {
    let mut chunk = [T::zeroed(); N];
    loop {
        let n = input.read(&mut chunk);
        apply(&mut chunk[..n]);
        output.write(&chunk[..n]);
        if n < N {
            break;
        }
    }
}

pub fn map<I: Sample, O: Sample, const N: usize>(
    mut input: impl Reader<I>,
    mut output: impl Writer<O>,
    mut map: impl FnMut(I) -> O,
) {
    let mut input_chunk = [I::zeroed(); N];
    let mut output_chunk = [O::zeroed(); N];
    loop {
        let n = input.read(&mut input_chunk);
        for i in 0..n {
            output_chunk[i] = map(input_chunk[i]);
        }
        output.write(&output_chunk[..n]);
        if n < N {
            break;
        }
    }
}

pub fn map_chunks<I: Sample, O: Sample, const N: usize>(
    mut input: impl Reader<I>,
    mut output: impl Writer<O>,
    mut map: impl FnMut([I; N]) -> [O; N],
) {
    let mut input_chunk = [I::zeroed(); N];
    loop {
        let n = input.read(&mut input_chunk);
        let output_chunk = map(input_chunk);
        output.write(&output_chunk[..n]);
        if n < N {
            break;
        }
    }
}

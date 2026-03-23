use crate::{
    io::{Reader, Writer},
    sample::Sample,
};

pub struct SliceReader<'d, T: Sample> {
    data: &'d [T],
    position: usize,
}

impl<'d, T: Sample> From<&'d [T]> for SliceReader<'d, T> {
    fn from(data: &'d [T]) -> Self {
        Self { data, position: 0 }
    }
}

impl<T: Sample> Reader<T> for SliceReader<'_, T> {
    fn read(&mut self, output: &mut [T]) -> usize {
        let pos = self.position;
        let remaining = self.data.len() - pos;
        let len = output.len().min(remaining);

        output[..len].copy_from_slice(&self.data[pos..pos + len]);
        self.position += len;

        len
    }
}

pub struct SliceWriter<'d, T: Sample> {
    data: &'d mut [T],
    position: usize,
}

impl<'d, T: Sample> From<&'d mut [T]> for SliceWriter<'d, T> {
    fn from(data: &'d mut [T]) -> Self {
        Self { data, position: 0 }
    }
}

impl<T: Sample> Writer<T> for SliceWriter<'_, T> {
    fn write(&mut self, input: &[T]) -> usize {
        let pos = self.position;
        let remaining = self.data.len() - pos;
        let len = self.data.len().min(remaining);

        self.data[pos..pos + len].copy_from_slice(&input[..len]);
        self.position += len;

        len
    }
}

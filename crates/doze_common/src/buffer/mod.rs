use heapless::Vec;

pub mod pod;

pub trait Buffer<T>: AsRef<[T]> + AsMut<[T]> {
    fn capacity(&self) -> usize;
}

impl<T> Buffer<T> for [T] {
    fn capacity(&self) -> usize {
        self.len()
    }
}

impl<T, const N: usize> Buffer<T> for [T; N] {
    #[inline(always)]
    fn capacity(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Buffer<T> for Vec<T, N> {
    fn capacity(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    use alloc::{boxed::Box, vec::Vec};

    use super::Buffer;

    impl<T> Buffer<T> for Box<[T]> {
        fn capacity(&self) -> usize {
            self.len()
        }
    }

    impl<T> Buffer<T> for Vec<T> {
        fn capacity(&self) -> usize {
            self.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{Reader, Writer};

    pub type Data = f32;
    pub const BUFFER_SIZE: usize = 8;

    pub fn read_write(mut buffer: impl Reader<Data> + Writer<Data>) {
        let input = [Data::MAX; BUFFER_SIZE];
        let mut output = [Data::default(); BUFFER_SIZE];

        buffer.write(&input);
        buffer.read(&mut output);

        assert_eq!(output, input);
    }

    pub fn write_overfill(mut buffer: impl Writer<Data>) {
        let input = [Data::MAX; BUFFER_SIZE + 1];

        assert_eq!(buffer.write(&input), BUFFER_SIZE);
    }

    pub fn write_on_full(mut buffer: impl Writer<Data>) {
        let null = [Data::default(); BUFFER_SIZE];
        let input = [Data::MAX; BUFFER_SIZE];

        buffer.write(&null);
        assert_eq!(buffer.write(&input), BUFFER_SIZE);
    }
}

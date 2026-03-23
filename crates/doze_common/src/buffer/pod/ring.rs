use bytemuck::Pod;

use crate::{
    buffer::Buffer,
    io::{Reader, ReaderExt, Writer, WriterExt},
};

#[derive(Debug)]
pub struct InnerRingBuffer<B> {
    buffer: B,
    write_position: usize,
    read_position: usize,
    count: usize,
}

impl<T: Copy, B: Buffer<T>> Reader<T> for InnerRingBuffer<B> {
    fn read(&mut self, output: &mut [T]) -> usize {
        let pos = self.read_position;
        let cap = self.buffer.capacity();
        let remaining = cap - pos;
        let len = output.len().min(self.count);

        let buffer = self.buffer.as_ref();

        if len <= remaining {
            output[..len].copy_from_slice(&buffer[pos..pos + len]);
        } else {
            output[..remaining].copy_from_slice(&buffer[pos..]);
            output[remaining..len].copy_from_slice(&buffer[..len - remaining]);
        }

        self.read_position = (pos + len) % cap;
        self.count -= len;

        len
    }
}

impl<T: Copy, B: Buffer<T>> Writer<T> for InnerRingBuffer<B> {
    fn write(&mut self, input: &[T]) -> usize {
        let pos = self.write_position;
        let cap = self.buffer.capacity();
        let len = input.len().min(cap);
        let remaining = (cap - pos).min(len);
        let buffer = self.buffer.as_mut();

        buffer[pos..pos + remaining].copy_from_slice(&input[..remaining]);

        let overflow = len - remaining;
        if overflow > 0 {
            buffer[..overflow].copy_from_slice(&input[remaining..len]);
        }

        self.write_position = (pos + len) % cap;

        if self.count + len > cap {
            self.read_position = (self.read_position + self.count + len - cap) % cap;
            self.count = cap;
        } else {
            self.count += len;
        }

        len
    }
}

impl<T: Copy, B: Buffer<T>> ReaderExt<T> for InnerRingBuffer<B> {
    fn remaining_read(&self) -> usize {
        self.count
    }
    fn restart_read(&mut self) {
        self.read_position = self.write_position;
        self.count = 0;
    }
}

impl<T: Copy, B: Buffer<T>> WriterExt<T> for InnerRingBuffer<B> {
    fn remaining_write(&self) -> usize {
        self.buffer.capacity() - self.count
    }
    fn restart_write(&mut self) {
        self.write_position = 0;
        self.count = 0;
    }
}

impl<T, B: Buffer<T>> AsRef<[T]> for InnerRingBuffer<B> {
    fn as_ref(&self) -> &[T] {
        self.buffer.as_ref()
    }
}

impl<T, B: Buffer<T>> AsMut<[T]> for InnerRingBuffer<B> {
    fn as_mut(&mut self) -> &mut [T] {
        self.buffer.as_mut()
    }
}

pub type RingBuffer<T, const N: usize> = InnerRingBuffer<[T; N]>;

impl<T: Pod, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self {
            buffer: [T::zeroed(); N],
            write_position: 0,
            read_position: 0,
            count: 0,
        }
    }
}

impl<T, const N: usize> Buffer<T> for RingBuffer<T, N> {
    #[inline(always)]
    fn capacity(&self) -> usize {
        N
    }
}

#[cfg(feature = "alloc")]
pub mod alloc {

    use alloc::boxed::Box;

    use crate::buffer::Buffer;

    use super::InnerRingBuffer;

    pub type RingBuffer<T> = InnerRingBuffer<Box<[T]>>;

    impl<T> RingBuffer<T> {
        pub fn new(capacity: usize) -> Self {
            Self {
                buffer: unsafe { Box::new_uninit_slice(capacity).assume_init() },
                write_position: 0,
                read_position: 0,
                count: 0,
            }
        }
    }

    impl<T> Buffer<T> for RingBuffer<T> {
        fn capacity(&self) -> usize {
            self.buffer.len()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::buffer::tests::{BUFFER_SIZE, *};

        #[test]
        fn test_read_write() {
            let buffer = RingBuffer::<Data>::new(BUFFER_SIZE);
            read_write(buffer);
        }

        #[test]
        fn test_write_overfill() {
            let buffer = RingBuffer::<Data>::new(BUFFER_SIZE);
            write_overfill(buffer);
        }

        #[test]
        fn test_write_on_full() {
            let buffer = RingBuffer::<Data>::new(BUFFER_SIZE);
            write_on_full(buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::tests::*;

    #[test]
    fn test_read_write() {
        let buffer = RingBuffer::<Data, BUFFER_SIZE>::default();
        read_write(buffer);
    }

    #[test]
    fn test_write_overfill() {
        let buffer = RingBuffer::<Data, BUFFER_SIZE>::default();
        write_overfill(buffer);
    }

    #[test]
    fn test_write_on_full() {
        let buffer = RingBuffer::<Data, BUFFER_SIZE>::default();
        write_on_full(buffer);
    }
}

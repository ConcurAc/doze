use bytemuck::Pod;

use crate::{
    buffer::Buffer,
    io::{Reader, ReaderExt, Writer, WriterExt},
};

const BUFFER_COUNT: usize = 2;

#[derive(Debug, Clone)]
pub struct InnerPingPongBuffer<B> {
    buffers: [B; BUFFER_COUNT],
    active: bool,
    read_position: usize,
    write_position: usize,
}

impl<B> InnerPingPongBuffer<B> {
    fn swap(&mut self) {
        self.active = !self.active;
        self.read_position = 0;
        self.write_position = 0;
    }
    fn active(&self) -> usize {
        self.active as usize
    }
    fn inactive(&self) -> usize {
        !self.active as usize
    }
}

impl<T: Copy, B: Buffer<T>> Reader<T> for InnerPingPongBuffer<B> {
    fn read(&mut self, output: &mut [T]) -> usize {
        let pos = self.read_position;
        let cap = self.buffers[self.active()].capacity();
        let remaining = cap - pos;
        let len = output.len().min(remaining);

        let buffer = self.buffers[self.active()].as_ref();

        output[..len].copy_from_slice(&buffer[pos..pos + len]);
        self.read_position += len;

        len
    }
}

impl<T: Copy, B: Buffer<T>> Writer<T> for InnerPingPongBuffer<B> {
    fn write(&mut self, input: &[T]) -> usize {
        let pos = self.write_position;
        let cap = self.buffers[self.inactive()].capacity();
        let remaining = cap - pos;
        let len = input.len().min(remaining);

        let buffer = self.buffers[self.inactive()].as_mut();

        buffer[pos..pos + len].copy_from_slice(&input[..len]);
        self.write_position += len;

        if cap == self.write_position {
            self.swap();
        }

        len
    }
}

impl<T: Copy, B: Buffer<T>> ReaderExt<T> for InnerPingPongBuffer<B> {
    fn restart_read(&mut self) {
        self.read_position = 0;
    }
    fn remaining_read(&self) -> usize {
        self.buffers.capacity() - self.read_position
    }
}

impl<T: Copy, B: Buffer<T>> WriterExt<T> for InnerPingPongBuffer<B> {
    fn restart_write(&mut self) {
        self.write_position = 0;
    }
    fn remaining_write(&self) -> usize {
        self.buffers[self.inactive()].capacity() - self.write_position
    }
}

pub type PingPongBuffer<T, const N: usize> = InnerPingPongBuffer<[T; N]>;

impl<T: Pod, const N: usize> Default for PingPongBuffer<T, N> {
    fn default() -> Self {
        Self {
            buffers: [[T::zeroed(); N]; BUFFER_COUNT],
            active: Default::default(),
            read_position: 0,
            write_position: 0,
        }
    }
}

#[cfg(feature = "alloc")]
pub mod alloc {
    use alloc::{boxed::Box, vec};

    use bytemuck::Pod;

    use super::InnerPingPongBuffer;

    pub type PingPongBuffer<T> = InnerPingPongBuffer<Box<[T]>>;

    impl<T: Pod> PingPongBuffer<T> {
        pub fn new(capacity: usize) -> Self {
            let buffer = vec![T::zeroed(); capacity].into_boxed_slice();
            Self {
                buffers: [buffer.clone(), buffer],
                active: Default::default(),
                read_position: 0,
                write_position: 0,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::buffer::tests::*;

        #[test]
        fn test_read_write() {
            let buffer = PingPongBuffer::<Data>::new(BUFFER_SIZE);
            read_write(buffer);
        }

        #[test]
        fn test_write_overfill() {
            let buffer = PingPongBuffer::<Data>::new(BUFFER_SIZE);
            write_overfill(buffer);
        }

        #[test]
        fn test_write_on_full() {
            let buffer = PingPongBuffer::<Data>::new(BUFFER_SIZE);
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
        let buffer = PingPongBuffer::<Data, BUFFER_SIZE>::default();
        read_write(buffer);
    }

    #[test]
    fn test_write_overfill() {
        let buffer = PingPongBuffer::<Data, BUFFER_SIZE>::default();
        write_overfill(buffer);
    }

    #[test]
    fn test_write_on_full() {
        let buffer = PingPongBuffer::<Data, BUFFER_SIZE>::default();
        write_on_full(buffer);
    }
}

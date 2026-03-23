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

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

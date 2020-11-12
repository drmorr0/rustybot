const ARRAY_SIZE: usize = 8;

pub struct MinArray<T> {
    data: [Option<(u32, T)>; ARRAY_SIZE],
    pub min: u32,
    pub len: usize,
}

impl<T> MinArray<T> {
    pub const fn new() -> MinArray<T> {
        MinArray {
            data: [None; ARRAY_SIZE],
            min: 0xffffff,
            len: 0,
        }
    }

    pub fn push(&mut self, key: u32, value: T) -> Result<(), T> {
        if self.len >= ARRAY_SIZE {
            return Err(value);
        }

        self.data[self.len] = Some((key, value));
        if key < self.min {
            self.min = key;
        }
        self.len += 1;
        Ok(())
    }

    pub fn take_less_than(&mut self, key: u32) -> MinArrayIterator<'_, T> {
        MinArrayIterator {
            array: self,
            index: 0,
            key,
        }
    }
}

pub struct MinArrayIterator<'a, T> {
    array: &'a mut MinArray<T>,
    index: usize,
    key: u32,
}

impl<'a, T> MinArrayIterator<'a, T> {
    fn swap_remove(&mut self) -> Option<(u32, T)> {
        self.array.len -= 1;
        self.array.data.swap(self.index, self.array.len);
        self.array.data[self.array.len].take()
    }

    fn recompute_min(&mut self) {
        self.array.min = 0xffffffff;
        for i in 0..ARRAY_SIZE {
            if let Some((key, _)) = self.array.data[i] {
                if key < self.array.min {
                    self.array.min = key;
                }
            }
        }
    }
}

impl<'a, T> Iterator for MinArrayIterator<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.array.len {
            if let Some((key, _)) = self.array.data[self.index] {
                if key < self.key {
                    let (_, value) = self.swap_remove().unwrap();
                    self.index -= 1;
                    return Some(value);
                }
            }
            self.index += 1;
        }
        self.recompute_min();
        None
    }
}

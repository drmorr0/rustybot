use core::{
    marker::PhantomData,
    ops::Index,
    slice::Iter,
};

const CAPACITY: usize = 8;

pub struct ConstVec<'a, T> {
    _lifetime: PhantomData<&'a T>,
    data: [Option<T>; CAPACITY],
    len: usize,
}

impl<'a, T> ConstVec<'a, T> {
    pub const fn new() -> ConstVec<'a, T> {
        ConstVec {
            _lifetime: PhantomData,
            data: [None; CAPACITY],
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    pub fn push(&mut self, el: T) -> Result<(), T> {
        if self.len >= CAPACITY {
            return Err(el);
        }
        self.data[self.len] = Some(el);
        self.len += 1;
        Ok(())
    }

    pub fn get(&self, id: usize) -> &Option<T> {
        &self.data[id]
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        let value = self.data[index].expect("index out of bounds");
        self.data[index] = self.data[self.len - 1].take();
        self.len -= 1;
        value
    }

    pub fn iter(&self) -> Iter<'_, Option<T>> {
        self.data.iter()
    }
}

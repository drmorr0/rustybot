// Code adapated from the rust-embedded-community async-on-embedded:
// https://github.com/rust-embedded-community/async-on-embedded/blob/master/async-embedded/src/alloc.rs

use core::mem::{
    self,
    MaybeUninit,
};

static mut MEMORY: [u8; 256] = [0xab; 256];

pub struct Allocator {
    len: usize,
    pos: usize,
    start: *mut u8,
}

static mut ALLOCATOR: Allocator = Allocator {
    len: unsafe { MEMORY.len() },
    pos: 0,
    start: unsafe { MEMORY.as_mut_ptr() },
};

impl Allocator {
    pub fn get() -> &'static mut Allocator {
        unsafe { &mut ALLOCATOR }
    }

    fn alloc<T>(&mut self) -> &'static mut MaybeUninit<T> {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();
        let new_pos = round_up(self.pos, align);
        if new_pos + size < self.len {
            self.pos = new_pos + size;
            unsafe { &mut *(self.start.add(new_pos) as *mut MaybeUninit<T>) }
        } else {
            panic!("out of memory");
        }
    }

    pub fn new<T>(&mut self, val: T) -> &'static mut T {
        let slot = self.alloc::<T>();
        unsafe {
            slot.as_mut_ptr().write(val);
            &mut *slot.as_mut_ptr()
        }
    }
}

// Round n to the nearest multiple of m
fn round_up(n: usize, m: usize) -> usize {
    let rem = n % m;
    if rem == 0 {
        n
    } else {
        (n + m) - rem
    }
}

// Code adapated from the rust-embedded-community async-on-embedded:
// https://github.com/rust-embedded-community/async-on-embedded/blob/master/async-embedded/src/alloc.rs

use core::mem::{
    self,
    MaybeUninit,
};

static mut ALLOCATOR_INIT: bool = false;
static mut MEMORY: [u8; 1024] = [0; 1024];

pub struct Allocator {
    len: usize,
    pos: usize,
    start: *mut u8,
}

impl Allocator {
    pub fn get() -> &'static mut Allocator {
        static mut ALLOCATOR: MaybeUninit<Allocator> = MaybeUninit::uninit();
        unsafe {
            if !ALLOCATOR_INIT {
                ALLOCATOR.as_mut_ptr().write(Allocator {
                    len: MEMORY.len(),
                    pos: 0,
                    start: MEMORY.as_mut_ptr(),
                });
            }
            &mut *(ALLOCATOR.as_mut_ptr() as *mut Allocator)
        }
    }

    fn alloc<T>(&mut self) -> &'static mut MaybeUninit<T> {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();
        let new_pos = round_up(self.pos, align);
        if new_pos + size < self.len {
            self.pos = new_pos + size;
            unsafe { &mut *(self.start.add(new_pos) as *mut MaybeUninit<T>) }
        } else {
            // OOM
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

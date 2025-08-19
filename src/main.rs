use std::alloc::{alloc, dealloc, Layout, handle_alloc_error};
use std::ptr;

struct ArenaBlock {
    next: *mut ArenaBlock,
    usage: usize,
    capacity: usize
}

struct Arena {
    start: *mut ArenaBlock,
    end: *mut ArenaBlock
}

impl ArenaBlock {
    pub fn new(capacity: usize) -> *mut ArenaBlock {
        let block_size = std::mem::size_of::<ArenaBlock>();
        let total_size = block_size + capacity;
        let align = std::mem::align_of::<ArenaBlock>();
        let layout = Layout::from_size_align(total_size, align).unwrap();

        unsafe {
            let ptr = alloc(layout) as *mut ArenaBlock;

            if ptr.is_null() {
                handle_alloc_error(layout);
            }

            (*ptr).next = ptr::null_mut();
            (*ptr).usage = 0;
            (*ptr).capacity = capacity;

            ptr
        }
    }
}

fn main() {
    println!("Hello, world!");
}

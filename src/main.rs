use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

const DEFAULT_CAPACITY: usize = 8 * 1024;

#[derive(Debug)]
pub enum ArenaError {
    OutOfMemory,
    InvalidSize,
    InvalidAlignment,
    NullPointer
}

pub type ArenaResult<T> = Result<T, ArenaError>;

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
    pub unsafe fn new(size: usize) -> ArenaResult<*mut ArenaBlock> {
        if size == 0 {
            return Err(ArenaError::InvalidSize);
        }
        let mut capacity = DEFAULT_CAPACITY;
        
        while size > capacity {
            capacity *= 2;
        }

        let block_size = std::mem::size_of::<ArenaBlock>();
        let total_size = block_size + capacity;
        let align = std::mem::align_of::<ArenaBlock>();
        let layout = Layout::from_size_align(total_size, align).map_err(|_| ArenaError::InvalidAlignment)?;

        unsafe {
            let ptr = alloc(layout) as *mut ArenaBlock;

            if ptr.is_null() {
                return Err(ArenaError::OutOfMemory);
            }

            (*ptr).next = ptr::null_mut();
            (*ptr).usage = 0;
            (*ptr).capacity = capacity;

            Ok(ptr)
        }
    }

    pub unsafe fn free(ptr: *mut ArenaBlock) -> ArenaResult<()> {
        if ptr.is_null() {
            return Err(ArenaError::NullPointer);
        }

        unsafe {
            let block_size = std::mem::size_of::<ArenaBlock>();
            let total_size = block_size + (*ptr).capacity;
            let align =  std::mem::align_of::<ArenaBlock>();
            let layout = Layout::from_size_align(total_size, align).unwrap();

            dealloc(ptr as *mut u8, layout);
        }

        Ok(())
    }
}

impl Arena {
    unsafe fn get_block_ptr(&self, block: *mut ArenaBlock) -> *mut u8 {
        unsafe {
            let ptr = block as *mut u8;
            let block_size = std::mem::size_of::<ArenaBlock>();    

            ptr.add(block_size)
        }
    }
}

impl Arena {
    pub fn new() -> Self {
        Self {
            start: ptr::null_mut(),
            end: ptr::null_mut(),
        }
    }

    pub fn alloc(&mut self, size: usize) -> ArenaResult<*mut u8> {
        if self.start.is_null() {
            unsafe {
                let block = ArenaBlock::new(size)?;
                self.start = block;
                self.end = block;
            }
        }

        unsafe {
            while (*self.end).usage + size > (*self.end).capacity && !(*self.end).next.is_null() {
                self.end = (*self.end).next;
            }

            if (*self.end).usage + size > (*self.end).capacity {
                let block = ArenaBlock::new(size)?;
                (*self.end).next = block;
                self.end = block;
            }

            let data_ptr = self.get_block_ptr(self.end);
            let ptr = data_ptr.add((*self.end).usage);
            (*self.end).usage += size;

            Ok(ptr)
        }

    }

    pub unsafe fn free(&mut self) -> ArenaResult<()> {
        unsafe {
            let mut current = self.start;

            while !current.is_null() {
                let next = (*current).next;
                ArenaBlock::free(current)?;
                current = next;
            }

            self.start = ptr::null_mut();
            self.end = ptr::null_mut();

            Ok(())
        }
    }
}

fn main() {
    let mut arena = Arena::new();

    let x = arena.alloc(10).unwrap();

    unsafe {
        (*x) = 2;

        println!("{:?}", (*x));
        match arena.free() {
            Ok(_) => println!("Freed"),
            Err(e) => println!("Error: {:?}", e)
        };
    }
}

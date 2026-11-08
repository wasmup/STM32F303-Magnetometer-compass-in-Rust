//! Volatile access to memory-mapped peripheral registers.
use core::ptr::{read_volatile, write_volatile};

pub fn read(addr: usize) -> u32 {
    unsafe { read_volatile(addr as *const u32) }
}
pub fn write(addr: usize, value: u32) {
    unsafe { write_volatile(addr as *mut u32, value) }
}
pub fn modify(addr: usize, clear: u32, set: u32) {
    write(addr, (read(addr) & !clear) | set);
}

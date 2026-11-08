//! Software I²C on PB6 (SCL) and PB7 (SDA).
use crate::register::{read, write};
use core::arch::asm;

const GPIOB: usize = 0x4800_0400;
const SCL: u32 = 1 << 6;
const SDA: u32 = 1 << 7;

fn pause() {
    for _ in 0..40 {
        unsafe { asm!("nop", options(nomem, nostack, preserves_flags)) }
    }
}
fn line(pin: u32, high: bool) {
    write(GPIOB + 0x18, if high { pin } else { pin << 16 });
    pause();
}
fn clock_high() -> bool {
    line(SCL, true);
    (0..1000).any(|_| read(GPIOB + 0x10) & SCL != 0)
}
fn start() -> bool {
    line(SDA, true);
    if !clock_high() {
        return false;
    }
    if read(GPIOB + 0x10) & SDA == 0 {
        // A reset during a transfer can leave a slave waiting for more clocks.
        for _ in 0..9 {
            line(SCL, false);
            if !clock_high() {
                return false;
            }
            if read(GPIOB + 0x10) & SDA != 0 {
                break;
            }
        }
        stop();
        if !clock_high() || read(GPIOB + 0x10) & SDA == 0 {
            return false;
        }
    }
    line(SDA, false);
    line(SCL, false);
    true
}
fn stop() {
    line(SDA, false);
    let _ = clock_high();
    line(SDA, true);
}
fn send(mut byte: u8) -> bool {
    for _ in 0..8 {
        line(SDA, byte & 0x80 != 0);
        if !clock_high() {
            return false;
        }
        line(SCL, false);
        byte <<= 1;
    }
    line(SDA, true);
    if !clock_high() {
        return false;
    }
    let ack = read(GPIOB + 0x10) & SDA == 0;
    line(SCL, false);
    ack
}
fn receive(more: bool) -> Option<u8> {
    let mut byte = 0;
    line(SDA, true);
    for _ in 0..8 {
        if !clock_high() {
            return None;
        }
        byte = (byte << 1) | u8::from(read(GPIOB + 0x10) & SDA != 0);
        line(SCL, false);
    }
    line(SDA, !more);
    if !clock_high() {
        return None;
    }
    line(SCL, false);
    line(SDA, true);
    Some(byte)
}
pub fn i2c_write(reg: u8, value: u8) -> bool {
    let ok = start() && send(0x1e << 1) && send(reg) && send(value);
    stop();
    ok
}
pub fn i2c_read(reg: u8, data: &mut [u8]) -> bool {
    let mut ok = start() && send(0x1e << 1) && send(reg) && start() && send((0x1e << 1) | 1);
    let len = data.len();
    for (i, byte) in data.iter_mut().enumerate() {
        if !ok {
            break;
        }
        if let Some(value) = receive(i + 1 < len) {
            *byte = value;
        } else {
            ok = false;
        }
    }
    stop();
    ok
}

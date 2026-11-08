#![no_std]
#![no_main]

mod board;
mod compass;
mod debounce;
mod i2c;
mod magnetometer;
mod register;

use core::arch::global_asm;
use core::panic::PanicInfo;
use debounce::Debouncer;

global_asm!(
    ".section .isr_vector,\"a\",%progbits",
    ".word _estack",
    ".word Reset_Handler"
);

const BUTTON_DEBOUNCE_SAMPLES: u8 = 3;
// Hard-iron offsets in raw sensor counts; replace after calibration.
const X_OFFSET: i32 = 0;
const Y_OFFSET: i32 = 0;

// Latest raw measurements, available for calibration in GDB.
#[no_mangle]
static mut LAST_X: i16 = 0;
#[no_mangle]
static mut LAST_Y: i16 = 0;
// Debugger status: 0 = detecting, 1 = read failed, 2 = zero vector, 3 = valid vector.
#[no_mangle]
static mut SENSOR_STATUS: u32 = 0;

fn set_sensor_status(status: u32) {
    unsafe { core::ptr::addr_of_mut!(SENSOR_STATUS).write_volatile(status) }
}

extern "C" {
    static mut _sdata: u32;
    static mut _edata: u32;
    static _sidata: u32;
    static mut _sbss: u32;
    static mut _ebss: u32;
}

#[no_mangle]
unsafe extern "C" fn Reset_Handler() -> ! {
    let mut src = core::ptr::addr_of!(_sidata);
    let mut dest = core::ptr::addr_of_mut!(_sdata);
    while dest < core::ptr::addr_of_mut!(_edata) {
        dest.write(src.read());
        dest = dest.add(1);
        src = src.add(1);
    }
    dest = core::ptr::addr_of_mut!(_sbss);
    while dest < core::ptr::addr_of_mut!(_ebss) {
        dest.write(0);
        dest = dest.add(1);
    }
    main()
}

fn main() -> ! {
    board::init();
    board::led_test();
    let mut sensor = None;
    let mut north = (1_i64, 0_i64);
    let mut button = Debouncer::new(false, BUTTON_DEBOUNCE_SAMPLES);
    let mut calibration_pending = false;
    loop {
        let pressed = board::user_button_pressed();
        if button.update(pressed) == Some(true) {
            board::led_test();
            calibration_pending = true;
        }
        if sensor.is_none() {
            sensor = magnetometer::configure_sensor();
            if sensor.is_none() {
                set_sensor_status(0);
            }
        }
        if let Some(device) = sensor {
            match magnetometer::sample(device) {
                Some((x, y)) => {
                    unsafe {
                        core::ptr::addr_of_mut!(LAST_X).write_volatile(x);
                        core::ptr::addr_of_mut!(LAST_Y).write_volatile(y);
                    }
                    let x = i64::from(x) - i64::from(X_OFFSET);
                    let y = i64::from(y) - i64::from(Y_OFFSET);
                    set_sensor_status(if x == 0 && y == 0 { 2 } else { 3 });
                    if calibration_pending && (x != 0 || y != 0) {
                        north = (x, y);
                        calibration_pending = false;
                        board::calibration_confirmation();
                    }
                    // Rotate the measured field into the captured north frame.
                    board::leds(compass::sector(
                        x * north.0 + y * north.1,
                        y * north.0 - x * north.1,
                    ));
                }
                None => {
                    set_sensor_status(1);
                    board::leds(0);
                    sensor = None;
                }
            }
        } else {
            board::leds(0);
        }
    }
}
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

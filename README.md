# STM32F303 Compass in Rust

A bare-metal compass for the STM32F3DISCOVERY board. It reads the onboard magnetometer over software I²C and displays direction using the eight onboard LEDs. The blue user button sets a reference to true north.

The firmware supports the LSM303DLHC and LSM303AGR magnetometers found on different board revisions. It uses direct register access and has no Rust crate dependencies. The current firmware has been tested on a board with an LSM303DLHC; the LSM303AGR path has not been verified on hardware.

## Hardware

- STM32F3DISCOVERY / STM32F303VCT6 with its onboard ST-LINK connected through USB Mini-B
- Onboard LSM303DLHC or LSM303AGR magnetometer; I²C uses PB6 (SCL) and PB7 (SDA)
- Blue user button on PA0 and eight LEDs on PE8–PE15
- Fitted ST-LINK jumpers for programming and debugging

No external wiring is required for the compass.

## Build and flash

Install the Rust toolchain, OpenOCD, and `make`, then add the ARM target:

```sh
rustup target add thumbv7em-none-eabihf
cargo build --release
make flash
```

Run these commands from the project directory with the board connected. `make flash` builds the release firmware, programs it through ST-LINK, verifies it, and resets the board. Flashing replaces the firmware already on the board. USB permissions may need to be configured for ST-LINK on your system.

The output ELF is `target/thumbv7em-none-eabihf/release/magnetometer-compass`. An ARM-capable GDB is also needed for debugging.

## Use the compass

1. After reset, the eight LEDs light in sequence from bit 0 to bit 7, for 100 ms each.
2. Hold the board level and point the **PE13 LED side** toward a known true-north reference.
3. Press and release the blue user button. Keep the board level and pointed north while the LED sequence runs again and until calibration is confirmed.
4. Once a valid reading is captured, all LEDs turn on for 100 ms and then turn off. The compass display resumes, initially lighting PE13.
5. Turn the board while keeping it level. The LEDs indicate the captured north direction relative to the board.

A reference is stored in RAM, so repeat the procedure after reset or power loss. Press the button again to replace it. Before a reference is captured, the display shows the magnetic field relative to the sensor axes.

A single LED represents a direction within about 15° of its center. Between approximately 15° and 30°, two adjacent LEDs light. The uncalibrated sensor-vector mapping is:

| Vector direction | LED | Bit |
| --- | --- | --- |
| +X | PE13 | 0 |
| +X, −Y | PE14 | 1 |
| −Y | PE15 | 2 |
| −X, −Y | PE8 | 3 |
| −X | PE9 | 4 |
| −X, +Y | PE10 | 5 |
| +Y | PE11 | 6 |
| +X, +Y | PE12 | 7 |

### Accuracy and calibration

The button sets a **direction reference**; it does not calibrate magnetic distortion. True-north alignment depends on the reference used when the button is pressed. Board tilt, nearby metal or magnets, hard-iron offsets, and soft-iron distortion can affect the result. Keep the board level and away from magnetic objects. LEDs may flicker near a sector boundary.

For a rough hard-iron correction, slowly rotate the level board through a full circle and record the minimum and maximum `LAST_X` and `LAST_Y` values in GDB. Set `X_OFFSET` and `Y_OFFSET` in [src/main.rs](src/main.rs) to each axis's midpoint, `(minimum + maximum) / 2`, then rebuild and flash. Accurate calibration may also require scale and cross-axis correction.

The DLHC is configured for its ±8.1 gauss range to reduce overflow in stronger fields, at the cost of sensitivity.

## Troubleshooting and debugging

All compass LEDs turn off if sensor detection or an I²C transaction fails, the DLHC reports overflow, or both corrected horizontal components are zero. The firmware retries sensor detection and attempts to recover a stuck I²C bus. If calibration is pending, it waits for a valid, nonzero reading before flashing all LEDs.

To inspect the sensor with GDB, start OpenOCD in one terminal:

```sh
openocd -f stm32f3.cfg
```

In another terminal, start an ARM-capable GDB and connect:

```sh
arm-none-eabi-gdb target/thumbv7em-none-eabihf/release/magnetometer-compass
```

```gdb
target extended-remote localhost:3333
monitor halt
print SENSOR_STATUS
print LAST_X
print LAST_Y
continue
```

Interrupt the program before reading again. `LAST_X` and `LAST_Y` contain the latest successful raw sample and remain unchanged after a failure. `SENSOR_STATUS` gives the most recently recorded state:

| Value | Meaning |
| --- | --- |
| 0 | Sensor detection has not succeeded |
| 1 | A sensor read failed or the DLHC reported overflow |
| 2 | The corrected horizontal reading was zero |
| 3 | A nonzero reading was obtained |

The firmware retries continuously, so status 1 can quickly change to 0 if detection then fails. A nonzero reading alone does not guarantee an accurate heading.

For VS Code debugging, install Cortex-Debug, OpenOCD, and `arm-none-eabi-gdb`, and ensure both commands are on `PATH`. Open this folder and press F5. The launch configuration builds and programs the firmware and stops at `Reset_Handler`; stop any separately running OpenOCD instance first.

## Implementation

- [src/main.rs](src/main.rs): startup, application loop, and hard-iron offsets
- [src/board.rs](src/board.rs): GPIO, button, LEDs, and LED timing
- [src/i2c.rs](src/i2c.rs): software I²C and bus recovery
- [src/magnetometer.rs](src/magnetometer.rs): sensor detection, configuration, and sampling
- [src/compass.rs](src/compass.rs): vector-to-LED mapping
- [src/debounce.rs](src/debounce.rs) and [src/register.rs](src/register.rs): button debouncing and register access

The compass calculation subtracts the configured offsets and rotates the vector relative to the captured reference. Integer comparisons select one or two LEDs without floating-point trigonometry. The firmware uses the default internal clock, busy-wait timing, and only a reset vector; interrupts and exception handlers are not configured.

The hardware-independent tests can be run on the host:

```sh
rustc --test src/debounce.rs -o /tmp/debounce-tests && /tmp/debounce-tests
rustc --test src/compass.rs -o /tmp/compass-tests && /tmp/compass-tests
```

## License

The project is licensed under the [MIT License](LICENSE), except [stm32f3.cfg](stm32f3.cfg), which is adapted from OpenOCD and licensed under [GPL-2.0-or-later](LICENSES/GPL-2.0-or-later.txt).

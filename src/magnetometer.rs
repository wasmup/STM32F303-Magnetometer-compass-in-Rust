//! Detection, configuration and sampling for the onboard magnetometer variants.
use crate::i2c::{i2c_read, i2c_write};

#[derive(Clone, Copy)]
pub enum Magnetometer {
    Dlhc,
    Agr,
}
pub fn configure_sensor() -> Option<Magnetometer> {
    let mut id = [0];
    if i2c_read(0x4f, &mut id) && id[0] == 0x40 {
        if i2c_write(0x60, 0x00) && i2c_write(0x62, 0x10) {
            return Some(Magnetometer::Agr);
        }
    } else if i2c_read(0x0a, &mut id)
        && id[0] == 0x48
        && i2c_write(0x00, 0x14)
        && i2c_write(0x01, 0xe0) // ±8.1 gauss to avoid saturation near strong fields.
        && i2c_write(0x02, 0x00)
    {
        return Some(Magnetometer::Dlhc);
    }
    None
}
pub fn sample(sensor: Magnetometer) -> Option<(i16, i16)> {
    let mut b = [0; 6];
    match sensor {
        Magnetometer::Dlhc if i2c_read(0x03, &mut b) => {
            let x = i16::from_be_bytes([b[0], b[1]]);
            let y = i16::from_be_bytes([b[4], b[5]]);
            let z = i16::from_be_bytes([b[2], b[3]]);
            if x == -4096 || y == -4096 || z == -4096 {
                None
            } else {
                Some((x, y))
            }
        }
        Magnetometer::Agr if i2c_read(0xe8, &mut b) => Some((
            i16::from_le_bytes([b[0], b[1]]),
            i16::from_le_bytes([b[2], b[3]]),
        )),
        _ => None,
    }
}

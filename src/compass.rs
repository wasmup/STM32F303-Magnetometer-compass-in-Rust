//! Convert a horizontal magnetic vector into an LED mask.

// LED mask, mirrored around the +X/-X axis. Between directions, light both LEDs.
pub fn sector(x: i64, y: i64) -> u8 {
    if x == 0 && y == 0 {
        return 0;
    }
    let ax = x.abs();
    let ay = y.abs();
    let x_led = if x >= 0 { 0 } else { 4 };
    let y_led = if y >= 0 { 6 } else { 2 };
    let diagonal = if x >= 0 && y >= 0 {
        7
    } else if x < 0 && y >= 0 {
        5
    } else if x < 0 {
        3
    } else {
        1
    };
    let (near, far, axis_led) = if ax >= ay {
        (ay, ax, x_led)
    } else {
        (ax, ay, y_led)
    };
    // tan(15°) ≈ 274/1024; tan(30°) ≈ 591/1024.
    if near * 1024 <= far * 274 {
        1 << axis_led
    } else if near * 1024 < far * 591 {
        (1 << axis_led) | (1 << diagonal)
    } else {
        1 << diagonal
    }
}

#[cfg(test)]
mod tests {
    use super::sector;

    #[test]
    fn maps_axes_and_diagonals_around_the_board() {
        for (x, y, bit) in [
            (1, 0, 0),
            (1, -1, 1),
            (0, -1, 2),
            (-1, -1, 3),
            (-1, 0, 4),
            (-1, 1, 5),
            (0, 1, 6),
            (1, 1, 7),
        ] {
            assert_eq!(sector(x, y), 1 << bit);
        }
    }

    #[test]
    fn handles_zero_and_between_sector_boundaries() {
        assert_eq!(sector(0, 0), 0);
        assert_eq!(sector(100, 20), 1);
        assert_eq!(sector(100, 40), 1 | (1 << 7));
        assert_eq!(sector(100, 100), 1 << 7);
    }
}

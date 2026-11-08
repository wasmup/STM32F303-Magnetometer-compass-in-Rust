/// Debounces a boolean input using consecutive samples, independent of hardware.
pub struct Debouncer {
    stable: bool,
    count: u8,
    samples: u8,
}

impl Debouncer {
    pub fn new(initial: bool, samples: u8) -> Self {
        assert!(samples > 0);
        Self {
            stable: initial,
            count: 0,
            samples,
        }
    }

    /// Returns the new stable state only when a transition has been accepted.
    pub fn update(&mut self, input: bool) -> Option<bool> {
        if input == self.stable {
            self.count = 0;
        } else {
            self.count += 1;
            if self.count == self.samples {
                self.stable = input;
                self.count = 0;
                return Some(input);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Debouncer;

    #[test]
    fn rejects_bounce_and_reports_each_stable_transition_once() {
        let mut button = Debouncer::new(false, 3);
        for input in [true, true, false, true, true] {
            assert_eq!(button.update(input), None);
        }
        assert_eq!(button.update(true), Some(true));
        assert_eq!(button.update(true), None);
        for input in [false, true, false, false] {
            assert_eq!(button.update(input), None);
        }
        assert_eq!(button.update(false), Some(false));
        assert_eq!(button.update(false), None);
    }

    #[test]
    fn one_sample_accepts_immediately() {
        let mut input = Debouncer::new(true, 1);
        assert_eq!(input.update(false), Some(false));
    }
}

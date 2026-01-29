use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

pub struct IdGenerator {
    counter: AtomicU32,
}

impl IdGenerator {
    pub fn new() -> IdGenerator {
        IdGenerator {
            counter: AtomicU32::new(0),
        }
    }
    pub fn get_next_id(&mut self) -> u32 {
        self.counter.fetch_add(1, Relaxed)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn begins_at_0() {
        let mut generator = IdGenerator::new();
        assert_eq!(generator.get_next_id(), 0)
    }
    #[test]
    fn increments_by_one() {
        let mut generator = IdGenerator::new();
        assert_eq!(generator.get_next_id(), 0);
        assert_eq!(generator.get_next_id(), 1)
    }
}

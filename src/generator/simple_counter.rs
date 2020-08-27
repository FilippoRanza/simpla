pub type AddrSize = u16;

pub struct SimpleCounter {
    counter: AddrSize,
}

impl SimpleCounter {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn count_one(&mut self) -> AddrSize {
        let output = self.counter;
        self.counter += 1;
        output
    }

    pub fn reset(&mut self) {
        self.counter = 0;
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn test_simple_counter() {
        let mut counter = SimpleCounter::new();
        for i in 0..10 {
            assert_eq!(i, counter.count_one());
        }
        assert_eq!(counter.counter, 10);
        counter.reset();
        assert_eq!(counter.counter, 0);
    }
}

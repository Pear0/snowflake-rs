
use generator::{BasicIDGenerator, IDGenerator};

#[derive(Debug)]
pub struct MultiIDGenerator {
    delegates: Vec<BasicIDGenerator>,
    last_index: usize,
}

impl MultiIDGenerator {
    pub fn from_generators(generators: Vec<BasicIDGenerator>) -> MultiIDGenerator {
        MultiIDGenerator {
            last_index: generators.len() - 1,
            delegates: generators,
        }
    }

    pub fn num_generators(&self) -> usize {
        self.delegates.len()
    }
}

impl IDGenerator for MultiIDGenerator {

    fn generate(&mut self) -> Option<i64> {
        for _ in 0..self.delegates.len() {
            self.last_index = (self.last_index + 1) % self.delegates.len();

            if let Some(id) = self.delegates[self.last_index].generate() {
                return Some(id)
            }
        }

        None
    }
}

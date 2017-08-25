
use super::system_millis;

pub trait IDGenerator {
    fn generate(&mut self) -> Option<i64>;
}

#[derive(Debug)]
pub struct BasicIDGenerator {
    start_epoch: i64,
    machine_id: u32,

    time: i64,
    sequence_id: i32,
    rollover_sequence_id: i32
}

impl BasicIDGenerator {
    pub fn new(start_epoch: i64, machine_id: u32) -> BasicIDGenerator {
        BasicIDGenerator {
            start_epoch: start_epoch,
            machine_id: machine_id,
            time: system_millis(),
            sequence_id: 0,
            rollover_sequence_id: 4095
        }
    }

    fn id_from_parts(epoch: i64, time: i64, machine_id: u32, sequence: i32) -> i64 {
        let relative_time = time - epoch;
        (relative_time << 22) | (machine_id & 0x3ff) as i64 | (sequence & 0xfff) as i64
    }
}

impl IDGenerator for BasicIDGenerator {
    fn generate(&mut self) -> Option<i64> {
        let millis = system_millis();

        if millis > self.time {
            self.time = millis;

            if self.sequence_id == -1 {
                self.sequence_id = (self.rollover_sequence_id + 1) & 0xfff
            } else {
                self.rollover_sequence_id = (self.sequence_id + 0xffe) & 0xfff
            }
        }

        if self.sequence_id == -1 {
            return None
        }

        let sequence_id = self.sequence_id;

        if self.sequence_id == self.rollover_sequence_id {
            self.sequence_id = -1
        } else {
            self.sequence_id = (self.sequence_id + 1) & 0xfff
        }

        Some(BasicIDGenerator::id_from_parts(self.start_epoch, self.time, self.machine_id, sequence_id))
    }
}

impl Iterator for BasicIDGenerator {
    type Item = i64;

    fn next(&mut self) -> Option<i64> {
        self.generate()
    }
}

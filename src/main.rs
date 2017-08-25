extern crate byteorder;
extern crate bytes;
extern crate futures;
extern crate time;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

mod generator;
mod server;

use generator::{IDGenerator};

pub fn system_millis() -> i64 {
    let timespec = time::get_time();
    timespec.sec * 1000 + (timespec.nsec as i64) / 1000 / 1000
}


fn bench() {
    let start = system_millis();

    let mut ids_generated = 0;
    let mut generator = IDGenerator::new(start, 0);

    while system_millis() - start < 5000 {

        loop {

            match generator.next() {
                Some(id) => {
                    if id % 100 == 0 {
                        println!("{}", id);
                    }
                    ids_generated += 1
                },
                None => break
            }

        }

    }

    let end = system_millis();

    println!("{:?} ids generated using {:?}", ids_generated, generator);
    println!("or {:?} ids ber millisecond over {:?}ms", ids_generated as f32 / (end - start) as f32, end - start)
}

fn main() {

    let generator = IDGenerator::new(system_millis(), 0);

    server::start_server(generator, "0.0.0.0:47322");

}

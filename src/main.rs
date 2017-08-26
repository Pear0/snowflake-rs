#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

extern crate byteorder;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate time;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

mod generator;
mod multi_generator;
mod server;

use clap::{App, Arg, ArgMatches, SubCommand};
use generator::{BasicIDGenerator, IDGenerator};
use multi_generator::MultiIDGenerator;
use std::process::exit;

// January 1, 2010 00:00:00 UTC
pub const DEFAULT_EPOCH: i64 = 1262304000;

pub fn system_millis() -> i64 {
    let timespec = time::get_time();
    timespec.sec * 1000 + (timespec.nsec as i64) / 1000 / 1000
}

fn construct_matches() -> ArgMatches<'static> {
    let common_args = vec![
        Arg::with_name("epoch")
            .short("t")
            .long("epoch")
            .value_name("SECONDS")
            .help("Set the epoch in seconds relative to the UNIX epoch")
            .long_help("This epoch is relative to the UNIX epoch which is useful because it has \
                       a more limited time range (about 70 years). The creation time of ids starts \
                       counting from the specified epoch. \
                       \nSpecifying 'now' as the epoch will cause the current time to be used. This can \
                       be useful for testing but is not recommended for general use because it will \
                       cause overlapping ids to be generated between runs.")
            .takes_value(true)
            .default_value("2010-1-1 00:00:00 UTC"),
        Arg::with_name("machine")
            .short("m")
            .long("machine")
            .value_name("ID")
            .help("Set the machine id to use when generating ids.")
            .long_help("Set the machine id to use when generating ids. \
            \nIf multiple ids are specified, they will all be used to increase maximum possible \
            throughput.")
            .takes_value(true)
            .multiple(true)
            .default_value("0"),
    ];

    App::new("snowflake_rs")
        .version(crate_version!())
        .author("Will Gulian <will@willgulian.com>")
        .about("Generates distributed ids")

        .subcommand(SubCommand::with_name("serve")
            .about("Opens a TCP server to request ids")
            .arg(Arg::with_name("bind")
                .short("b")
                .long("bind")
                .value_name("HOST:PORT")
                .help("Provide a custom ip address and port to bind")
                .takes_value(true)
                .default_value("0.0.0.0:47322")
            )
            .args(&common_args)
        )
        .subcommand(SubCommand::with_name("bench")
            .about("Runs a simple benchmark to estimate theoretical id throughput")
            .args(&common_args)
        )

        .get_matches()
}

fn resolve_epoch(matches: &ArgMatches<'static>) -> i64 {
    if matches.occurrences_of("epoch") > 0 {
        match matches.value_of("epoch").unwrap() {
            "now" => system_millis(),
            id => match id.parse::<i64>() {
                Ok(t) => t * 1000,
                Err(_) => {
                    error!("{:?} is not a valid integer epoch", id);
                    exit(1)
                }
            },
        }
    } else {
        DEFAULT_EPOCH * 1000
    }
}

fn create_generator_from_common(matches: &ArgMatches<'static>) -> MultiIDGenerator {
    let epoch = resolve_epoch(matches);

    debug!("Epoch: {:?}", epoch);

    let machine_ids: Vec<_> = matches.values_of("machine").unwrap()
        .map(|x| x.parse::<u32>().unwrap())

        .collect();

    debug!("IDs: {:?}", machine_ids);

    let generators: Vec<_> = machine_ids.into_iter().map(|id| BasicIDGenerator::new(epoch, id)).collect();

    MultiIDGenerator::from_generators(generators)
}

fn main_serve(matches: &ArgMatches<'static>) {
    let bind = matches.value_of("bind").unwrap();
    let generator = create_generator_from_common(matches);

    server::start_server(generator, bind);
}

fn main_bench(matches: &ArgMatches<'static>) {
    let mut ids_generated = 0;
    let mut generator = create_generator_from_common(matches);

    let start = system_millis();

    while system_millis() - start < 5000 {
        for _ in 0..100 {
            match generator.generate() {
                Some(id) => {
                    if id % 100 == 0 {
                        //println!("{}", id);
                    }
                    ids_generated += 1
                },
                None => break
            }
        }
    }

    let end = system_millis();

    println!("{:?} ids generated", ids_generated);

    let ids_per_ms = ids_generated as f32 / (end - start) as f32;

    println!("or {:?} ids ber millisecond over {:?}ms", ids_per_ms, end - start);
    println!("or {:?} ids per machine id per ms", ids_per_ms / generator.num_generators() as f32);
}

fn main() {
    env_logger::init().unwrap();

    let matches = construct_matches();

    match matches.subcommand() {
        ("serve", Some(matches)) => main_serve(matches),
        ("bench", Some(matches)) => main_bench(matches),
        _ => error!("No subcommand specified"),
    }


}

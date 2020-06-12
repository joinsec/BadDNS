use std::fs::File;
use std::process::{exit, id};

use crate::logger::{init, LoggingLevel};
use clap::{ App, Arg };
use colored::*;


#[derive(Clone, Debug)]
pub struct Config {
    pub domain_file: String,
    pub sub_domain_dict: String,
    pub depth_dict_file: String,
    pub output_file: String,
    pub depth: usize,
    pub worker: usize,
    pub retry: usize,
    pub use_mem: f64,
    pub pid: u32,
}

impl Config {
    pub fn new() -> Self {
        let matches = App::new("BadDNS")
            .version("1.0.1")
            .author("Link <link.messagebox@gmail.com>")
            .about("Subdomain detection system")
            .arg(Arg::with_name("target")
                .short("t")
                .long("target")
                .value_name("FILE")
                .required(true)
                .help("Set the target file")
                .takes_value(true))
            .arg(Arg::with_name("sub")
                .short("s")
                .long("sub")
                .value_name("FILE")
                .help("Set up the sub dictionary file. Default: domaindict-170W.txt")
                .takes_value(true))
            .arg(Arg::with_name("depth")
                .short("d")
                .long("depth")
                .help("Set up the depth dictionary file. Default: depthdict.txt")
                .value_name("FILE")
                .takes_value(true))
            .arg(Arg::with_name("output")
                .short("o")
                .value_name("FILE")
                .long("output")
                .help("Setting result save file. Default: baddns-output.json")
                .takes_value(true))
            .arg(Arg::with_name("layer")
                .long("layer")
                .short("l")
                .help("Set query level domain. Default: 1")
                .takes_value(true))
            .arg(Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Set log display verbosity"))
            .arg(Arg::with_name("worker")
                .short("w")
                .long("worker")
                .help("Set worker number. Default: 500")
                .takes_value(true))
            .arg(Arg::with_name("mem")
                .short("m")
                .long("mem")
                .help("Memory utilization. Default: 50%")
                .takes_value(true))
            .get_matches();

        match matches.occurrences_of("v") {
            1 => {init(LoggingLevel::Normal); ()},
            2 => {init(LoggingLevel::Critical); ()},
            3 => {init(LoggingLevel::Debug); {}},
            0 | _ => {init(LoggingLevel::Off); ()},
        }

        let target_file = matches.value_of("target").unwrap().to_string();
        verify_file(target_file.as_str());

        let sub_file = matches.value_of("sub").unwrap_or("domaindict-170W.txt").to_string();
        verify_file(sub_file.as_str());

        let depth_file = matches.value_of("depth").unwrap_or("depthdict.txt").to_string();
        verify_file(depth_file.as_str());

        let output_file = matches.value_of("output").unwrap_or("baddns-output.json").to_string();

        let layer: usize = matches.value_of("layer").unwrap_or("1").parse().unwrap();

        let worker: usize = matches.value_of("worker").unwrap_or("500").parse().unwrap();

        let retry: usize = matches.value_of("retry").unwrap_or("3").parse().unwrap();

        let use_mem: f64 = matches.value_of("mem").unwrap_or("0.5").parse().unwrap();

        let pid = id();

        Self {
            domain_file: target_file,
            sub_domain_dict: sub_file,
            depth_dict_file: depth_file,
            output_file,
            depth: layer,
            worker,
            retry,
            use_mem,
            pid

        }
    }

    pub fn get_target_file(&self) -> &String{
        &self.domain_file
    }

    pub fn get_sub_dict_file(&self) -> &String {
        &self.sub_domain_dict
    }

    pub fn get_depth_dict_file(&self) -> &String {
        &self.depth_dict_file
    }

    pub fn get_output_file(&self) -> &String {
        &self.output_file
    }

    pub fn get_depth(&self) -> usize {
        self.depth
    }

    pub fn get_worker(&self) -> usize {
        self.worker
    }

    pub fn get_retry(&self) -> usize {
        self.retry
    }

    pub fn get_use_mem(&self) -> f64 {
        self.use_mem
    }

    pub fn get_pid(&self) -> u32 {
        self.pid
    }
}

fn verify_file(name: &str) {
    match File::open(name) {
        Ok(_) => {}
        Err(e) => {
            error!("{} {}", name, e);
            info!("For more information try --help");
            exit(1);
        }
    }
}

pub fn show_logo() {
    println!("{}", " ____            _ ____  _   _ ____".red());
    println!("{}", "| __ )  __ _  __| |  _ \\| \\ | / ___|".red());
    println!("{}", "|  _ \\ / _` |/ _` | | | |  \\| \\___ \\".red());
    println!("{}", "| |_) | (_| | (_| | |_| | |\\  |___) |".red());
    println!("{}", "|____/ \\__,_|\\__,_|____/|_| \\_|____/".red());
    println!("\n{}", "                            ".bold().red());
}

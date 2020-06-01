use std::thread;
use std::fs::File;
use std::process::exit;
use std::io::{BufReader, BufRead};
use std::sync::mpsc::{Receiver, Sender};

use indicatif::ProgressBar;

use crate::check_handler::GenItem;
use crate::gen_handler::Item;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueueMessage {
    Job(Item),
    Gen(GenItem),
    Clear,
    Terminate,
    Sleep,
    Break,
}


pub enum Statue {
    Check,
    // Gen,
    Query,
    Write,
    Unwrite,
    Terminate,
    Querys,
    Writes,
    Checks,
    TargetCount,
}

pub fn mem_total() -> f64 {
    let f = match File::open("/proc/meminfo") {
        Ok(f) => f,
        Err(e) => {
            println!("[util] Failure mem_total,Currently only supports Linux. msg:{:?}", e);
            exit(1);
        }
    };

    get_use_mem4field(f, "MemTotal:")

}

pub fn get_use_mem4pid(pid: u32) -> f64 {
    let file_path = "/proc/".to_owned() + pid.to_string().as_str() + "/status";
    let f = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            println!("[util] Failure get_use_mem4pid ,Currently only supports Linux. msg:{:?}", e);
            exit(1)
        }
    };
    get_use_mem4field(f, "VmRSS:")
}

pub fn get_use_mem4field(f: File, field: &str) -> f64 {
    let reader = BufReader::new(f);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.is_empty() {continue}
                let mut s = line.split_whitespace();
                if s.next().unwrap() == field {
                    return s.next().unwrap().parse().unwrap()
                }
            }
            Err(_) => {}
        }
    }
    println!("[util] Failure get_use_mem4field,Currently only supports Linux");
    exit(1);
}

pub fn state_management(
    statistical_recv: Receiver<Statue>,
    items: Sender<QueueMessage>,
    pb: ProgressBar,
    sub_len: usize,
    target_len: usize) -> thread::JoinHandle<()> {

    thread::spawn(move || {

        // all
        let mut checks = 0;
        let mut querys = 0;
        let mut _writes = 0;
        let mut target = 0;

        // complete
        let mut check= 0;
        let mut query = 0;
        let mut write =0;
        let mut un_write = 0;

        let mut terminate = false;
        let mut terminate_send_statue = false;

        let mut alls = ((target_len * sub_len) + target_len) as u64;
        pb.set_length(alls);

        for i in statistical_recv {
            match i {
                Statue::TargetCount => {
                    target += 1;
                    alls = (((target + target_len) * sub_len) + target_len)as u64;
                    //  (10 + 5)* 1751385 + 10
                    pb.set_length(alls)
                }
                Statue::Checks => {
                    checks +=1
                }
                Statue::Querys => {
                    querys += 1;
                }
                Statue::Writes => {
                    _writes += 1
                }
                Statue::Write => {
                    write += 1
                }
                Statue::Check => {
                    check += 1
                }
                Statue::Query => {
                    query +=1;
                    pb.inc(1);
                }
                Statue::Unwrite => {
                    un_write +=1;
                }
                Statue::Terminate => {
                    terminate = true;
                }
            }

            // debug!("check {:?}, checks {:?}, query {:?}, querys {:?}, write {:?}, writes {:?}, un-write {:?},alls {:?}",
            //          check, checks, query, querys, write, writes, un_write, alls);

            let write = write + un_write;

            if querys.eq(&check) && querys.eq(&checks) && querys.eq(&query) && querys.eq(&write) && querys.eq(&alls){
                if terminate {
                    debug!("[mem_util] Break");
                    pb.finish();
                    break
                }

                if !terminate_send_statue {
                    match items.send(QueueMessage::Terminate) {
                        Ok(_) => {
                            debug!("[mem_util] QueueMessage::Terminate");
                        },
                        Err(_) => {
                            error!("[mem_util] send Terminate");
                        }
                    };
                    terminate_send_statue = true;
                }
            }

        }

        debug!("[mem_util] break end");
    })

}

use std::thread;
use std::io::Write;
use std::process::exit;
use std::fs::OpenOptions;
use std::sync::mpsc::{Receiver, Sender};

use serde_json;
use serde_derive::{Serialize, Deserialize};

use crate::mem_util::Statue;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResultsSubDomain {
    pub subdomain: String,
    pub collect: Vec<String>,
}

pub fn write_event(filename: String,
                   results_recv: Receiver<ResultsSubDomain>,
                   statistical_send: Sender<Statue>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        debug!("[write_event]Start save result...");

        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename) {
            Ok(f) => f,
            Err(e) => {
                error!("[write_event] {}", e);
                exit(1);
            }
        };

        file.write_all(b"[\n").unwrap();

        debug!("[write_event] recv...");

        for q in results_recv {
            match statistical_send.send(Statue::Write) {
                Ok(_) => {},
                Err(_) => {
                    error!("[write_handler] send Statue::Write");
                }
            };
            serde_json::to_writer(&mut file, &q).unwrap();
            drop(q);
            file.write_all(b",\n").unwrap();
        };

        file.write_all(b"{}\n]").unwrap();

        debug!("[write_event] save results end");
    })
}
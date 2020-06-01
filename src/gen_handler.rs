use std::thread;
use std::fs::File;
use std::time::Duration;
use std::io::{BufReader, BufRead};
use std::sync::mpsc::{Receiver, Sender};

use crate::mem_util::{mem_total, get_use_mem4pid, Statue, QueueMessage};


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    pub sub: String,
    pub subdomain: String,
    pub depth: usize,
    pub collect: Option<Vec<String>>,
}

pub fn gen_event(gen_recv: Receiver<QueueMessage>, query_send: Sender<QueueMessage>,
                 sub_list: Vec<String>, target: Vec<String>, statistical_send: Sender<Statue>, pid: u32, use_mem:f64) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        debug!("Start gen for target list ");
        let statistical_send_target = statistical_send.clone();

        for t in target.iter() {
            match statistical_send_target.send(Statue::Querys) {
                Ok(_) => {},
                Err(_) => {
                    error!("[gen_handler] gen_event send Statue::Querys ");
                }
            };

            let item = gen_item(t.as_str(), "", 0);
            query_send.send(QueueMessage::Job(item)).unwrap();
        }


        let statistical_send = statistical_send.clone();

        for q in gen_recv {
            match q {
                QueueMessage::Gen(item) => {
                    for sub in &sub_list {
                        if supper(use_mem, pid) {
                            thread::sleep(Duration::new(2, 0))
                        }
                        statistical_send.send(Statue::Querys).unwrap();
                        let item = gen_item(item.domain.as_ref(), sub, item.depth);
                        query_send.send(QueueMessage::Job(item)).unwrap();
                    };
                }
                QueueMessage::Terminate => {
                    query_send.send(QueueMessage::Terminate).unwrap();
                    break
                }
                _ => {}
            }
        }
        debug!("[gen_handler] break end");
    })
}

pub fn init_target(filename: &str) -> Vec<String> {
    let mut target = Vec::new();

    let f = File::open(filename).unwrap();

    let reader = BufReader::new(f);

    for item in reader.lines() {
        match item {
            Ok(item) => {
                let item = item.trim().to_lowercase() + ".";
                target.push(item);
            }
            Err(e) => {
                warn!("[item] init target {:?}", e)
            }
        }
    }

    target
}

fn supper(use_mem: f64, pid: u32) -> bool{
    if (get_use_mem4pid(pid) / mem_total()) * 0.75 >= use_mem {
        debug!("supper");
        return true
    }
    false
}

fn gen_item(domain: &str, sub: &str, depth: usize) -> Item {
    let subdomain =joint_subdomain(domain, sub);

    Item {
        sub: sub.to_owned(),
        subdomain,
        depth,
        collect: None
    }
}

fn joint_subdomain(domain: &str, sub: &str) -> String {
    if sub.is_empty() {
        return domain.to_string()
    }
    let subdomain = sub.to_owned() + "." + domain;
    subdomain
}
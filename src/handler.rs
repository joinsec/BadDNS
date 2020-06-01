use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::process::exit;

use pool_rs::pool::ThreadPool;

use crate::query::{query_event, Protocol};
use crate::gen_handler::Item;
use crate::mem_util::{Statue, QueueMessage};

// use rand::Rng;

pub fn subdomain_query_event(query_recv: Receiver<QueueMessage>, check_send: Sender<QueueMessage>,
                             worker: usize, retry: usize, statistical_send: Sender<Statue>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        debug!("Start subdomain query");

        let pool = ThreadPool::new(worker);

        let statistical_send = statistical_send.clone();

        for q in query_recv {
            let statistical_send = statistical_send.clone();
            let check = check_send.clone();

            match q {
                QueueMessage::Job(item) => {
                    let subdomain = item.subdomain.to_owned();
                    let sub = item.sub.to_owned();
                    let depth = item.depth.to_owned();

                    pool.execute(move || {
                        drop(item);
                        match statistical_send.send(Statue::Query) {
                            Ok(_) => {},
                            Err(_) => {
                                error!("[handler] send Statue::Query");
                            }
                        };
                        let mut collect = Vec::new();
                        query_event(subdomain.as_ref(), &mut collect, retry, Protocol::TCP);

                        let check_item = Item {
                            sub: sub.to_owned(),
                            subdomain: subdomain.to_owned(),
                            depth: depth.to_owned(),
                            collect: Some(collect)
                        };
                        match check.send(QueueMessage::Job(check_item.to_owned())) {
                            Ok(_) => {},
                            Err(e) => {
                                error!("[handler] send check Job {:?} {:?}", e, check_item);
                                exit(1)
                            }
                        };


                        match statistical_send.send(Statue::Checks) {
                            Ok(_) => {},
                            Err(_) => {
                                error!("[handler] send Statue::Checks {:?}", check_item);
                            }
                        }
                        drop(check_item);
                    });
                }
                QueueMessage::Terminate => {
                    debug!("[handler] Terminate");
                    drop(q);
                    match check.send(QueueMessage::Terminate) {
                        Ok(_) => { break;},
                        Err(_) => {
                            error!("[handler] send check_send");
                        }
                    };
                }
                _ => {}
            }
        }

        debug!("[handler] break end");
    })
}
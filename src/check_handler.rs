use std::thread;
use std::sync::mpsc::{Sender, Receiver};

use crate::mem_util::{QueueMessage, Statue};
use crate::write_handler::ResultsSubDomain;
use crate::dict::Dict;
use crate::wildcards::Wildcards;


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenItem {
    pub domain: String,
    pub depth: usize
}


pub fn check_event(check_recv: Receiver<QueueMessage>,
                   gen_send: Sender<QueueMessage>,
                   result_send: Sender<ResultsSubDomain>,
                   depth: usize, depth_dict: Dict,
                   w: Wildcards,
                   statistical_send: Sender<Statue> ) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let result_send = result_send.clone();
        let w = w.clone();
        let depth_dict = depth_dict.clone();
        let gen_send = gen_send.clone();
        let statistical_send = statistical_send.clone();

        for q in check_recv {
            match q {
                QueueMessage::Job(item) => {
                    // set check statistical
                    match statistical_send.send(Statue::Check) {
                        Ok(_) => {},
                        Err(_) => {
                            error!("[check_handler] send Statue::Check");
                        }
                    };
                    
                    // Add: 
                    // Subdomain generation is required regardless of
                    // whether the primary domain has a result or the result is whitelist

                    if item.depth.eq(&0) {
                        let items = GenItem {
                            depth: item.depth +1,
                            domain: item.subdomain.to_owned(),
                        };

                        // send gen_send
                        match gen_send.send(QueueMessage::Gen(items)) {
                            Ok(_) => {}
                            Err(_) => {
                                error!("[check_handler] send gen_send");
                            }
                        }
                    };

                    // check collect is None or vec list
                    if check_collect(&item.collect) {
                        let collect = item.collect.clone().unwrap();

                        // check item depth
                        if item.depth.eq(&0) {
                            let t  = gen_result(&item.subdomain, &collect);
                            
                            match result_send.send(t) {
                                Ok(_) => {statistical_send.send(Statue::Writes).unwrap();}
                                Err(_) => {
                                    error!("[check_handler] send Statue::Writes");
                                }
                            }
                        } else {
                            if check_wildcards(&w, &collect) {
                                let t = gen_result(&item.subdomain, &collect);
                                match result_send.send(t) {
                                    Ok(_) => {statistical_send.send(Statue::Writes).unwrap();}
                                    Err(_) => {
                                        error!("[check_handler] send Statue::Writes");
                                    }
                                }

                                if check_depth(item.depth, depth) {
                                    if check_depth_dict(&item.sub, &depth_dict) {
                                        let items = GenItem {
                                            domain: item.subdomain.to_owned(),
                                            depth: item.depth + 1,
                                        };

                                        match gen_send.send(QueueMessage::Gen(items)) {
                                            Ok(_) => {
                                                statistical_send.send(Statue::TargetCount).unwrap();
                                                drop(item)
                                            }
                                            Err(_) => {
                                                error!("[check_handler] send gen_send");
                                            }
                                        }
                                    }
                                }
                            }
                            else {
                                // set Unwrite statue
                                match statistical_send.send(Statue::Unwrite) {
                                    Ok(_) => {},
                                    Err(_) => {
                                        error!("[check_handler] send Statue::Unwrite");
                                    }
                                }
                            }
                        }


                    } else {
                        // set Unwrite statue
                        match statistical_send.send(Statue::Unwrite) {
                            Ok(_) => {},
                            Err(_) => {
                                error!("[check_handler] send Statue::Unwrite");
                            }
                        }
                    }
                }
                QueueMessage::Terminate => {
                    debug!("[check_handler] Terminate");
                    match statistical_send.send(Statue::Terminate) {
                        Ok(_) => {
                            debug!("[check_handler ] statistical_send Terminate");
                            break
                        },
                        Err(_) => {
                            error!("[check_handler] send Statue::Terminate");
                        }
                    };
                }
                _ => {}
            }
        }
        drop(result_send);
    })
}

fn gen_result(domain: &str, collect: &Vec<String>) -> ResultsSubDomain {
    ResultsSubDomain {
        subdomain: domain.trim_end_matches('.').to_owned(),
        collect: collect.to_vec()
    }
}

fn check_depth(item_depth: usize, depth: usize) -> bool {
    if item_depth + 1 > depth { return false }
    true
}

fn check_collect(collect: &Option<Vec<String>>) -> bool {
    if collect.is_some() && !collect.as_ref().unwrap().is_empty(){
        return true
    }
    false
}

fn check_wildcards(wildcards: &Wildcards, collect: &Vec<String>) -> bool {

    if wildcards.lists.is_empty() {
        return true
    }

    for i in collect {
        if wildcards.is_exist(i) {
            return false
        }
    }
    true
}

fn check_depth_dict(sub: &str, depth_dict: &Dict) -> bool {
    if depth_dict.len().eq(&0) {
        return false
    }
    depth_dict.is_exist(&sub.to_string())
}

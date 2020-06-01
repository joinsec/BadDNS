use std::thread;
use std::str::FromStr;
use std::time::Duration;
use std::sync::mpsc::channel;

use pool_rs::pool::ThreadPool;
use trust_dns_client::rr::{Name, DNSClass, RecordType};
use trust_dns_client::tcp::TcpClientConnection;
use trust_dns_client::client::{SyncClient, Client};

use crate::query::{ALL_DNS_SERVER, query_response_handler};


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Wildcards {
    pub lists: Vec<String>,
}

impl Wildcards {
    pub fn new() -> Self {
        Self {
            lists: Vec::new()
        }
    }

    pub fn set_item(&mut self, item: String) {
        self.lists.push(item)
    }

    pub fn is_exist(&self, item: &String) -> bool {
        self.lists.contains(item)
    }

    pub fn get_list(&mut self) -> Vec<String> {
        self.lists.sort();
        self.lists.dedup();
        self.lists.clone()
    }

    pub fn len(self) -> usize {
        self.lists.len()
    }
}

pub fn wildcards_event(domains: Vec<String>, depth: Vec<String>, worker: usize, w: &mut Wildcards) {

    let (check_send, check_recv) = channel();
    let pool = ThreadPool::new(worker);

    let mut w_depth =vec!["d6p4lfaojz.".to_string()];
    for d in depth {
        let sub = "d6p4lfaojz.".to_owned() + d.trim()+ ".";
        w_depth.push(sub);
    }


    let init_wildcards = thread::spawn(move || {
        for domain in domains {

            for d in w_depth.clone() {

                let subdomain = d.to_owned() + domain.trim();
                let check_send = check_send.clone();

                pool.execute(move || {
                    let mut collect = Vec::new();
                    query_wildcards(subdomain.as_str(), &mut collect);
                    check_send.send(collect).unwrap();
                })
            }
        }
    });

    for collect in check_recv {
        for item in collect {
            w.set_item(item);
        }
    }
    init_wildcards.join().unwrap();

    info!("{}", "Complete the whitelist generation operation");
}


fn query_wildcards(subdomain: &str, collect: &mut Vec<String>) {
    let name = match Name::from_str(subdomain) {
        Ok(n) => {n},
        Err(_) => {return},
    };

    for dns in ALL_DNS_SERVER.iter() {
        let client = match TcpClientConnection::with_timeout(dns.parse().unwrap(), Duration::from_secs_f32(5.0)) {
            Ok(c) => SyncClient::new(c),
            Err(e) => {
                warn!("[wildcards] tcp_connection. msg: {:?}", e);
                continue
            }
        };

        match client.query(&name, DNSClass::IN, RecordType::A) {
            Ok(q) => {
                query_response_handler(q, collect)
            }
            Err(_) => {}
        }

        match client.query(&name, DNSClass::IN, RecordType::CNAME) {
            Ok(q) => {
                query_response_handler(q, collect)
            }
            Err(_) => {}
        }
    }

}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_exist() {
        let mut w = Wildcards::new();
        w.lists = vec!["170.33.0.251".to_string(), "31.13.83.16.t".to_string(),"www.58coin.com".to_string(), "www.58ex.com".to_string()];
        assert_eq!(w.is_exist(&"1.1.1.1".to_string()), false);
        assert_eq!(w.is_exist(&"170.33.0.251".to_string()), true);
        assert_eq!(w.is_exist(&"www.58coin.com".to_string()), true);
    }

}
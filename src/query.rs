use std::time::Duration;
use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;


use rand::Rng;
use trust_dns_client::op::DnsResponse;
use trust_dns_client::udp::UdpClientConnection;
use trust_dns_client::tcp::TcpClientConnection;
use trust_dns_client::client::{SyncClient, Client};
use trust_dns_client::rr::{Name, RecordType, DNSClass};

pub static ALL_DNS_SERVER: [&str;21] = [
    "8.8.8.8:53", "8.8.4.4:53", "1.1.1.1:53", "1.0.0.1:53", "1.1.1.2:53", "1.0.0.2:53", "1.1.1.3:53",
    "1.0.0.3:53" ,"208.67.222.222:53", "208.67.220.220:53", "8.26.56.26:53", "8.20.247.20:53",
    "208.244.0.4:53", "216.146.35.35:53","216.146.36.36:53", "195.46.39.39:53","195.46.39.40:53",
    "206.189.193.106:53","84.200.69.80:53","84.200.70.40:53", "144.76.103.143:53"];

pub static UDP_SERVER: [&str; 11] = ["208.67.222.222:53", "208.67.220.220:53", "8.26.56.26:53",
    "8.20.247.20:53",  "208.244.0.4:53", "216.146.35.35:53","216.146.36.36:53","195.46.39.39:53",
    "195.46.39.40:53", "84.200.69.80:53","84.200.70.40:53"];

pub static TCP_SERVER: [&str; 8] = ["8.8.8.8:53", "8.8.4.4:53", "1.1.1.1:53", "1.0.0.1:53",
    "1.1.1.2:53", "1.0.0.2:53", "1.1.1.3:53", "1.0.0.3:53"];


#[derive(Clone, Eq, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
}

#[derive(Clone, Eq, PartialEq)]
pub enum RecordTypes {
    CNAME,
    A,
    AAAA,
}

pub fn query_event(subdomain: &str, collect: &mut Vec<String>, retry: usize, protocol: Protocol) {
    debug!("[query] query_event Start query");
    match Name::from_str(subdomain) {
        Ok(n) => {
            // A
            query_main(&n, RecordTypes::A, protocol.clone(), collect, retry, 0);

            // CNAME
            query_main(&n, RecordTypes::CNAME, protocol, collect, retry, 0);
        }
        Err(e) => {
            warn!("[query] query_event. msg: {:?}", e.kind());
        }
    }
}

fn query_main(subdomain: &Name, t:RecordTypes, protocol: Protocol,
              collect: &mut Vec<String>, retry: usize, count: usize) {

    let rt = match t {
        RecordTypes::CNAME => RecordType::CNAME,
        RecordTypes::A => RecordType::A,
        RecordTypes::AAAA => RecordType::AAAA
    };

    match protocol {
        Protocol::TCP => {
            let client = tcp_connection();
            match client.query(subdomain, DNSClass::IN, rt) {
                Ok(q) => {
                    query_response_handler(q, collect);
                }
                Err(_) => {
                    thread::sleep(Duration::from_secs_f32(0.3));
                    // let count= count + 1;
                    query_main(subdomain, t, protocol, collect, retry, count)
                }
            }
        }
        Protocol::UDP => {
            let client = udp_connection(count);
            match client.query(subdomain, DNSClass::IN, rt) {
                Ok(q) => {
                    query_response_handler(q, collect);
                }
                Err(_) => {
                    thread::sleep(Duration::from_secs_f32(0.3));
                    let count  = count + 1;

                    if count == retry {
                        query_main(subdomain, t.clone(), Protocol::TCP, collect, retry, 0);
                    }
                    query_main(subdomain, t, protocol, collect, retry, count);

                    // let r = e.kind();
                    // match r {
                    //     ClientErrorKind::Timeout => {
                    //         let count  = count + 1;
                    //         if count > retry {
                    //             query_main(subdomain, t.clone(), Protocol::TCP, collect, retry, 0);
                    //         }
                    //         query_main(subdomain, t, protocol, collect, retry, count);
                    //
                    //     }
                    //     _ => {
                    //         info!("[query] udp query_main. {} msg: {:?} {}",subdomain, e, count);
                    //
                    //         // query_main(subdomain, t, protocol, collect, retry, count)
                    //     }
                }
            }
        }
    }
}

pub fn query_response_handler(q: DnsResponse, collect: &mut Vec<String>) {
    if !q.is_empty() {
        for i in q.answers() {
            if i.rr_type().is_cname() {
                match i.rdata().as_cname() {
                    Some(a) => {collect.push(a.to_string().trim_end_matches('.').to_string())}
                    None => {}
                }
            }
            if i.rr_type().is_ip_addr() {
                match i.rdata().to_ip_addr() {
                    Some(ip) => {collect.push(ip.to_string())}
                    None => {}
                }
            }
        }
    }
}

fn udp_connection(count: usize) -> SyncClient<UdpClientConnection> {
    match UdpClientConnection::with_timeout(rand_dns_server(count), Duration::from_secs_f32(60.0)) {
        Ok(c) => SyncClient::new(c),
        Err(e) => {
            warn!("[query] udp_connection. msg: {:?}", e.kind());
            udp_connection(count)
        }
    }
}

fn tcp_connection() -> SyncClient<TcpClientConnection> {
    match TcpClientConnection::with_timeout(rand_tcp_dns_server(), Duration::from_secs_f32(120.0)) {
        Ok(c) => SyncClient::new(c),
        Err(e) => {
            warn!("[query] tcp_connection. msg: {:?}", e);
            tcp_connection()
        }
    }
}

fn rand_dns_server(count: usize) -> SocketAddr {
    let mut rng = rand::thread_rng();
    if count < 2 {
        let num = rng.gen_range(0, UDP_SERVER.len());

        match (UDP_SERVER[num]).parse() {
            Ok(p) => return p,
            Err(e) => {
                warn!("[query] rand_dns_server, msg: {:?}", e);
                rand_dns_server(count);
            }
        }
    }
    let num = rng.gen_range(0, ALL_DNS_SERVER.len());
    match (ALL_DNS_SERVER[num]).parse() {
        Ok(p) => return p,
        Err(e) => {
            warn!("[query] rand_dns_server, msg: {:?}", e);
            rand_dns_server(count)
        }
    }
}


fn rand_tcp_dns_server() -> SocketAddr {
    let mut rng = rand::thread_rng();

    let num = rng.gen_range(0, TCP_SERVER.len());
    match (TCP_SERVER[num]).parse() {
        Ok(p) => p,
        Err(_) => rand_tcp_dns_server()
    }
}

pub fn gen_subdomain(sub: &str, domain: &str) -> Option<Name> {
    let subdomain = sub.to_owned() + domain.trim();
    match Name::from_str(&subdomain) {
        Ok(n) => Some(n),
        Err(_) => {None}
    }
}

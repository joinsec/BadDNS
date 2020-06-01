#[macro_use] extern crate log;

#[doc(hidden)] #[macro_use] pub mod logger;

use yansi::Paint;

use baddns::cli::{show_logo, Config};
use baddns::dict::Dict;
use baddns::wildcards::{Wildcards, wildcards_event};
use indicatif::ProgressBar;
use std::sync::mpsc::channel;
use baddns::write_handler::write_event;
use baddns::check_handler::check_event;
use baddns::gen_handler::{gen_event, init_target};
use baddns::handler::subdomain_query_event;
use baddns::mem_util::state_management;

fn main() {
    show_logo();
    let c = Config::new();

    launch_info!("[1/5] {} Configured for sub dict",  Paint::masked("🔧"));
    let sub_dict = Dict::new(c.get_sub_dict_file());
    info!("{} Load sub dict: {}", Paint::masked("✅ "), sub_dict.len());

    launch_info!("[2/5] {} Configured for depth dict", Paint::masked("🔧"));
    let depth = Dict::new(c.get_depth_dict_file());
    info!("{} Load depth dict: {}", Paint::masked("✅ "), depth.len());

    launch_info!("[3/5] {} Create a thread pool", Paint::masked("🔧"));
    info!("{} Create {} threads", Paint::masked("✅ "), c.get_worker());

    launch_info!("[4/5] {} Initialization target", Paint::masked("🔧"));
    let target = init_target(c.get_target_file());
    info!("{} target count: {}",Paint::masked("✅ "), target.len());

    launch_info!("[5/5] {} Initialization whitelist", Paint::masked("🔧"));
    let mut whitelist = Wildcards::new();
    wildcards_event(target.clone(), depth.clone().get_dict(), c.get_worker(), &mut whitelist);
    let w = whitelist.clone().get_list();
    if w.len() > 0 {
        info!("{} Collected {} whitelist records, Show whitelist:", Paint::masked("✅ "), w.len());
        println!("{:?}", w);
    } else {
        info!("{} No whitelist", Paint::masked("✅️ "));
    }

    launch_info!("{} ignition...", Paint::masked("🚀 "));

    // create progressbar
    let pb = ProgressBar::new(0);

    // create channel
    let (gen_send, gen_recv) = channel();
    let (result_send, result_recv) = channel();
    let (check_send, check_recv) = channel();

    let (query_send, query_recv) = channel();

    // statue
    let (statistical_send, statistical_recv) = channel();

    let mem_statue = state_management(statistical_recv, gen_send.clone(), pb, sub_dict.len(), target.len());
    let query_handler = subdomain_query_event(query_recv, check_send, c.get_worker(), c.get_retry(), statistical_send.clone());

    let write_handler = write_event(c.get_output_file().to_owned(), result_recv, statistical_send.clone());

    let check_handler = check_event(check_recv, gen_send, result_send,
                                    c.get_depth(), depth, whitelist, statistical_send.clone());

    let gen_handler = gen_event(gen_recv, query_send, sub_dict.get_dict(), target, statistical_send, c.get_pid(), c.get_use_mem());



    mem_statue.join().unwrap();
    debug!("mem_statue end");
    write_handler.join().unwrap();
    debug!("write_handler end");

    match check_handler.join() {
        Ok(_) => {},
        Err(e) => {debug!("adda {:?}", e)}
    };
    debug!("check_handler end");
    gen_handler.join().unwrap();
    debug!("gen_handler end");
    query_handler.join().unwrap();
    debug!("query_handler end");



}

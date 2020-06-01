#[macro_use] extern crate log;

#[doc(hidden)] #[macro_use] pub mod logger;
pub mod cli;
pub mod dict;
pub mod query;
pub mod mem_util;
pub mod handler;
pub mod wildcards;
pub mod gen_handler;
pub mod write_handler;
pub mod check_handler;
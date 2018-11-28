#![recursion_limit = "128"]
#![allow(proc_macro_derive_resolution_fallback)]

extern crate log;
#[macro_use]
extern crate serenity;
extern crate config;
extern crate kankyo;
extern crate rand;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sys_info;
extern crate typemap;
extern crate calc;

pub mod commands;
pub mod core;

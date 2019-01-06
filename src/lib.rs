#![recursion_limit = "128"]
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
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
extern crate meval;
extern crate sentry;
//#[macro_use]
// The above macro_use is commented out as it is currently not used, but will be in the future.
extern crate failure;
pub mod commands;
pub mod core;

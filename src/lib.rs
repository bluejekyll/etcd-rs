extern crate rustc_serialize;
//extern crate chrono;
extern crate hyper;
extern crate url;


#[macro_use]
extern crate log;

pub mod etcd;

#[cfg(test)]
mod tests;

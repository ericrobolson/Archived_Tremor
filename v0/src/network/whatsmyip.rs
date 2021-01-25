/*
extern crate hyper;

use hyper::header::Connection;
use hyper::Client;
use std::io::Read;

pub fn extern_ip() -> String {
    let client = Client::new();
    let mut res = client
        .get("http://ipinfo.io/ip/")
        .header(Connection::close())
        .send()
        .unwrap();
}
*/

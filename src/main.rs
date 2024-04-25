use std::str::from_utf8;
use crate::future::{ServerFuture};

mod future;

#[tokio::main]
async fn main() {
    loop {
        let process = async { ServerFuture::new("127.0.0.1:3562").expect(
            "Cannot create a socket reader future").await };
        let val = process.await;
        println!("Read: {}", from_utf8(val.as_slice()).unwrap());
    }
}

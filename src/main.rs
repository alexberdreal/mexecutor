use std::str::from_utf8;
use crate::future::{ServerFuture};

mod future;

#[tokio::main]
async fn main() {
    loop {
        let val = ServerFuture::new("0.0.0.0:3562").expect(
            "Cannot create a socket reader future").await;
        println!("Read: {}", from_utf8(val.as_slice()).unwrap());
    }
}

use std::{future::Future, io, pin::Pin, task::{Context, Poll}};
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use futures::FutureExt;

pub struct ServerFuture {
    listener: TcpListener,
    readers : Vec<SocketReaderFuture>
}

pub struct SocketReaderFuture {
    stream : TcpStream,
    buffer : Vec<u8>
}

impl SocketReaderFuture {
    pub fn new(stream : TcpStream) -> Self {
        SocketReaderFuture { stream: stream, buffer: vec![0; 1024] }
    }
}

impl Future for SocketReaderFuture {
    type Output = Vec<u8>;

    fn poll(self: Pin<&mut Self>, _ : &mut Context<'_>) -> Poll<Self::Output> {
        let mut_self = self.get_mut();
        let mut cur_len = mut_self.buffer.len();
        loop {
            let mut temp_buf : [u8; 1024] = [0; 1024];
            match mut_self.stream.read(&mut temp_buf) {
                Ok(0) => {
                    let buf_clone = mut_self.buffer.clone();
                    mut_self.buffer.clear();
                    return Poll::Ready(buf_clone)
                }
                Ok(n) => {
                    mut_self.buffer.extend_from_slice(&temp_buf);
                    cur_len += n;
                }
                Err(ref err) => {
                    // TODO: error handling
                    return Poll::Pending
                }
            }
        }
    }
}

impl ServerFuture {
    pub fn new(addr : &str) -> Result<Self, io::Error> {
        match TcpListener::bind(addr) {
            Ok(listener) => {
                // Set non blocking mode for the tcp stream
                listener.set_nonblocking(true).expect(
                    "Failed to set non blocking mode on a socket"
                );

                Ok(ServerFuture { listener: listener, readers: Vec::with_capacity(16) })
            }
            Err(err) => {
                Err(err)
            }
        }
    }
}

impl Future for ServerFuture {
    type Output = Vec<u8>;

    fn poll(self: Pin<&mut Self>, ctx : &mut Context<'_>) -> Poll<Self::Output> {
        let mut_self = self.get_mut();
        for future in &mut mut_self.readers {
            let res = future.poll_unpin(ctx);
            if let Poll::Ready(_) = res {
                ctx.waker().wake_by_ref();
                return res;
            }
        }
        match mut_self.listener.accept() {
            Ok((stream, _)) => {
                let cap = mut_self.readers.capacity();
                if mut_self.readers.len() + 1 == cap {
                    mut_self.readers.reserve(cap * 2);
                }
                let reader_future = SocketReaderFuture::new(stream);
                mut_self.readers.push(reader_future);
            }
            Err(ref err) => {
                if err.kind() != io::ErrorKind::WouldBlock {
                    println!("Cannot accept: {}", err.to_string());
                }
            }
        }
        ctx.waker().wake_by_ref();
        Poll::Pending
    }
}
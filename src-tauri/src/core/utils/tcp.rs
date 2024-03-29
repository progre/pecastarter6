use std::num::NonZeroU16;

use anyhow::Result;
use log::trace;
use tokio::{
    io::copy,
    join,
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

use super::failure::Failure;

pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<TcpStream, Failure> {
    for _ in 0..5 {
        let result = TcpStream::connect(&addr).await;
        match result {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                log::trace!("connect error: {}", e);
                log::trace!("retry...");
            }
        }
    }
    Err(Failure::Fatal("Connection error".into()))
}

pub async fn pipe(incoming: TcpStream, outgoing: TcpStream) {
    let (mut incoming_read, mut incoming_write) = incoming.into_split();
    let (mut outgoing_read, mut outgoing_write) = outgoing.into_split();
    trace!("Start piping");
    let (result1, result2) = join!(
        copy(&mut incoming_read, &mut outgoing_write),
        copy(&mut outgoing_read, &mut incoming_write),
    );
    trace!("End piping {:?} {:?}", result1, result2);
}

pub async fn find_free_port() -> Option<NonZeroU16> {
    match TcpListener::bind("0.0.0.0:0").await {
        Ok(listener) => {
            let addr_opt = listener.local_addr();
            drop(listener);
            match addr_opt {
                Ok(addr) => Some(NonZeroU16::new(addr.port()).unwrap()),
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::net::SocketAddr;

// simply echo what client send
async fn handle(mut client: TcpStream, addr: SocketAddr) {
    let mut buff = [0;1024];

    let length = match client.read(&mut buff).await {
        Err(_) | Ok(0) => {
            log::warn!("{addr}: Did not send a message");
            return
        },
        Ok(length) => length,
    };

    let msg = String::from_utf8_lossy(&buff[0..length-1]);
    log::info!("{addr}: {msg}");

    match client.write(format!("=> {msg}\n").as_bytes()).await {
        Ok(0) => log::warn!("{addr}: Server sent an empty string"),
        Ok(_) => (),
        Err(e) => log::error!("{addr}: {e}")
    };
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} [ip] [port]", args[0]);
        return
    }

    simple_logger::init().unwrap();
    let (ip, port) = (&args[1], &args[2]);
    let server = TcpListener::bind(format!("{ip}:{port}")).await.expect("couldnt start server");

    log::info!("Listening on {ip}:{port}");
    loop {
        let (client_stream, client_addr) = server.accept().await.unwrap();
        log::info!("New client from {client_addr}");
        tokio::spawn(async move {
            handle(client_stream, client_addr).await;
            log::info!("Lost client from {client_addr}");
        });
    }
}

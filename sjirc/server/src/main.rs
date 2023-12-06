use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::net::SocketAddr;

async fn handle(mut client: TcpStream, addr: SocketAddr) {
    let mut buff: [u8; 10] = [0;10];
    client.write("Hello, World !\n".as_bytes()).await.unwrap();
    let length = client.read(&mut buff).await.unwrap();
    log::info!("{addr}: {}", String::from_utf8_lossy(&buff[0..length-1]));
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} [ip] [port]", args[0]);
    }

    simple_logger::init().unwrap();
    let (ip, port) = (&args[1], &args[2]);
    let server = TcpListener::bind(format!("{ip}:{port}")).await.expect("couldnt start server");

    log::info!("Listening on {ip}:{port}");
    loop {
        let (client_stream, client_addr) = server.accept().await.unwrap();
        log::info!("New client from {client_addr}");
        tokio::spawn(async move {
            handle(client_stream, client_addr.clone()).await;
            log::info!("Lost client from {client_addr}");
        });
    }
}

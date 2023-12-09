use std::error::Error;
use std::io::{stdout,Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, AsyncBufReadExt, ReadHalf, stdin};
use tokio::net::TcpStream;

async fn read_from_server(mut reader: ReadHalf<TcpStream>) -> Result<(), Box<dyn Error>> {
    let mut msg = [0;1024];
    loop {
        match reader.read(&mut msg).await {
            Ok(n) => {
                let message = String::from_utf8_lossy(&msg[0..n]);
                print!("{}", message);
                stdout().flush().expect("couldnt flush stdout");
            }
            Err(e) => return Err(Box::new(e)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} [ip] [port]", args[0]);
        return Ok(())
    }
    let (ip, port) = (&args[1], &args[2]);
    println!("Listening on {ip}:{port}");

    let stream = TcpStream::connect(format!("{ip}:{port}")).await?;
    let (reader, mut writer) = tokio::io::split(stream);

    tokio::spawn(async move {
        let _ = read_from_server(reader).await;
    });

    let mut user_input = String::new();
    let mut stdin_reader = BufReader::new(stdin());
    loop {
        stdin_reader.read_line(&mut user_input).await.expect("couldnt read line from user");
        writer.write_all(user_input.as_bytes()).await?;
        user_input.clear();
    }
}

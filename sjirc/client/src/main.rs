use std::error::Error;
use std::io;
use std::io::stdin;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} [ip] [port]", args[0]);
    }
    let (ip, port) = (&args[1], &args[2]);
    println!("Listening on {ip}:{port}");

    let mut stream = TcpStream::connect(format!("{ip}:{port}")).await?;

    //stream.write_all(b"/NICKNAME").await?;
    //stream.write_all(b"hello world!").await?;

    //Ok(())

    loop {
        // Wait for the socket to be readable
        stream.readable().await?;
        let mut msg = vec![0; 1024];
        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_read(&mut msg) {
            Ok(n) => {
                msg.truncate(n);
                let string = String::from_utf8(msg).expect("Our bytes should be valid utf8");
                println!("{:?}", string);
                let mut s = String::new();
                stdin()
                    .read_line(&mut s)
                    .expect("Did not enter a correct string");
                stream.write_all(s.as_bytes()).await?;
                //break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
}

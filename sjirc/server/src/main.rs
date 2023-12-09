use tokio::net::{TcpStream, TcpListener, tcp::WriteHalf};
use tokio::io::{AsyncWriteExt, BufReader, AsyncReadExt, AsyncBufReadExt};
use tokio::sync::broadcast::{channel, Sender, Receiver};
use std::net::SocketAddr;

const MAX_CLIENT: usize = 10;
const USERNAME_LENGTH: usize = 10;

/// IrcMessage is the object passed through client (within the server)
/// it simply contains the `message` to send to all users and the `username`
/// linked to that message
#[derive(Clone,Debug)]
struct IrcMessage {
    username: String,
    message : String
}

/// get_username is the first (and only for now) setup the client go through
/// the client gets a prompt to enter its username
/// if the user quit, the connection is closed
/// otherwise the client is greeted
async fn get_username(client: &mut TcpStream) -> Option<String> {
    let mut buff = [0;USERNAME_LENGTH];

    client.write_all("your username: ".as_bytes()).await.unwrap();
    let length = match client.read(&mut buff).await {
        Err(_) | Ok(0) => return None,
        Ok(length) => length,
    };

    let username = String::from_utf8_lossy(&buff[0..length-1]);
    client.write_all(format!("Welcome {username} !\n").as_bytes()).await.unwrap();

    Some(username.into())
}

/// handle a client sending a command (send message, go away, quit, ...)
/// for now it just send the message to all users
async fn client_sent_command(line: String, username: &String, sender: Sender<IrcMessage>) {
    let line = line.trim();
    log::info!("{username}: [broadcast::send] {}", line);

    if sender.send( IrcMessage{ username: username.clone(), message: line.into() } ).is_err() {
        log::error!("{username}: [broadcast::send] couldnt send message: {line}"); 
    }
    log::info!("{username}: [broadcast::send] message sent with success");
}

/// handle a message coming from broadcast
/// (a command which require to send data to users in channel)
async fn broadcast_incoming<'a>(irc_struct: IrcMessage, username: &String, writer: &mut WriteHalf<'a>) {
    log::debug!("{username}: [broadcast::recv] {:?}", irc_struct);

    if irc_struct.username != *username {
        let message = format!("[{}]: {}\n", irc_struct.username, irc_struct.message);
        if writer.write_all(message.as_bytes()).await.is_err() {
            log::warn!("{username}: [broadcast::recv] couldnt send message to {username}");
        }
    }
}

/// handle a client
/// - get the username
/// - select send/recv from broadcast
async fn handle(mut client: TcpStream, addr: SocketAddr, sender: Sender<IrcMessage>, mut receiver: Receiver<IrcMessage>) {
    let Some(username) = get_username(&mut client).await else {
        log::warn!("{addr}: [username] not provided");
        return;
    };
    log::info!("{addr}: [username] {username}");

    let (reader, mut writer) = client.split();
    let mut reader = BufReader::new(reader);

    // main client loop
    let mut line = String::new();
    loop {
        tokio::select! {
            // receive from client send to broadcast
            length = reader.read_line(&mut line) => {
                if length.is_err() || length.unwrap() == 0 {
                    log::warn!("{username}: [broadcast::send] message is empty");
                    return
                }
                client_sent_command(line.clone(), &username, sender.clone()).await;
            }

            // receive from broadcast send to client
            irc_struct = receiver.recv() => {
                if irc_struct.is_err() {
                    log::warn!("{username}: [broadcast::recv] received corrupted IrcMessage");
                    continue
                }
                broadcast_incoming(irc_struct.unwrap(), &username, &mut writer).await;
            }

        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} [ip] [port]", args[0]);
        return
    }

    simple_logger::init_with_env().unwrap();
    let (ip, port) = (&args[1], &args[2]);
    let (sender, _) = channel::<IrcMessage>(MAX_CLIENT); // setup channel to share messages between clients
    let server = TcpListener::bind(format!("{ip}:{port}")).await.expect("couldnt start server");

    log::info!("Listening on {ip}:{port}");
    loop {
        let (client_stream, client_addr) = server.accept().await.unwrap();
        log::info!("{client_addr}: New Connection");

        let (client_sender, client_receiver) = (sender.clone(), sender.subscribe());
        log::debug!("number of peer(s): {}", sender.receiver_count());

        tokio::spawn(async move {
            handle(client_stream, client_addr, client_sender.clone(), client_receiver).await;
            log::info!("{client_addr}: Lost connection");
            log::debug!("number of peer(s): {}", client_sender.receiver_count());
        });
    }
}

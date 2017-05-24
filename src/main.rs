use std::{env, io, net, process};
use std::io::prelude::*;
use std::error::Error;

enum Command {
    Listen,
    Send,
}

const USAGE: &'static str = "Usage: mccat <listen | send> address port";

type AppResult<T> = Result<T, Box<Error>>;

fn main() {
    if let Err(err) = run() {
        let _ = writeln!(io::stderr(), "{}", err);
        process::exit(1);
    }
}


fn run() -> AppResult<()> {
    let (cmd, multiaddr, port) = parse_cmdline()?;

    if !multiaddr.is_multicast() {
        Err(io::Error::new(io::ErrorKind::InvalidInput,
                           format!("{} is not a multicast address", multiaddr)))?;
    }

    match cmd {
        Command::Listen => {
            let sock = match multiaddr {
                net::IpAddr::V4(addr) => {
                    let sockaddr: net::SocketAddr = (net::Ipv4Addr::from(0), port).into();
                    let sock = net::UdpSocket::bind(sockaddr)?;
                    sock.join_multicast_v4(&addr, &0.into())?;
                    println!("Listening on {}", net::SocketAddr::from((addr, port)));
                    sock
                }
                net::IpAddr::V6(addr) => {
                    let sockaddr: net::SocketAddr = (net::Ipv6Addr::from([0u8; 16]), port).into();
                    let sock = net::UdpSocket::bind(&sockaddr)?;
                    sock.join_multicast_v6(&addr, 0)?;
                    println!("Listening on {}", net::SocketAddr::from((addr, port)));
                    sock
                }
            };
            let mut buf = [0u8; 16384];
            loop {
                let (len, src) = sock.recv_from(&mut buf)?;
                let data = &buf[..len];
                println!("{} said: {}", src, String::from_utf8_lossy(data));
            }
        }
        Command::Send => {
            let sock = match multiaddr {
                net::IpAddr::V4(addr) => {
                    let sock = net::UdpSocket::bind((net::Ipv4Addr::from(0), 0))?;
                    sock.connect((addr, port))?;
                    sock
                }
                net::IpAddr::V6(addr) => {
                    let sock = net::UdpSocket::bind((net::Ipv6Addr::from([0u8; 16]), 0))?;
                    sock.connect((addr, port))?;
                    sock
                }
            };
            let mut buf = [0u8; 16384];
            let mut stdin = io::stdin();
            loop {
                let len = stdin.read(&mut buf)?;
                if len == 0 {
                    return Ok(());
                }
                let mut data = &buf[..len];
                if let Some(&b'\n') = data.last() {
                    // chomp
                    data = &data[..len - 1];
                }
                sock.send(data)?;
            }
        }
    }
}

fn parse_cmdline() -> AppResult<(Command, net::IpAddr, u16)> {
    let mut args = env::args().skip(1);
    let cmd = args.next()
        .ok_or(io::Error::new(io::ErrorKind::InvalidInput, USAGE))?;
    let addr = args.next()
        .ok_or(io::Error::new(io::ErrorKind::InvalidInput, USAGE))?;
    let port = args.next()
        .ok_or(io::Error::new(io::ErrorKind::InvalidInput, USAGE))?;

    let cmd = match &*cmd {
        "listen" => Command::Listen,
        "send" => Command::Send,
        _ => {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid cmd specified").into())
        }
    };

    let addr: net::IpAddr = addr.parse()?;
    let port: u16 = port.parse()?;

    Ok((cmd, addr, port))
}

use std::io::prelude::*;
use std::io::{self, Read};
use std::net::TcpListener;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut connections = Vec::new();
    let mut team_black_health = Vec::<u8>::new();
    let listener = TcpListener::bind("127.0.0.1:1370")?;
    let ready = false;
    while !ready {
        println!("here0");
        let connection = listener.accept()?;
        connections.push(connection.0);

        let mut ready_checks = Vec::new();
        for mut stream in &connections {
            println!("here");
            stream.write(b"ready?")?;
            stream.set_nonblocking(true)?;
            println!("here2");
            if let Some(response) = non_blocking_read(stream.try_clone()?) {
                println!("here3");
                let response = String::from_utf8_lossy(&response);
                if response == "ready" {
                    println!("Ready!");
                    ready_checks.push(true);
                    stream.write(b"Leggo.")?;
                } else {
                    ready_checks.push(false);
                }
            }
            println!("here4");
        }
    }
    println!("here5");

    Ok(())
}

fn non_blocking_read(mut stream: TcpStream) -> Option<[u8; 1024]> {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(_) => return Some(buffer),
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => return None,
        Err(e) => panic!("encountered IO error: {}", e),
    };
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    stream.write(&buffer)?;
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    Ok(())
}

use std::env::args_os;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    // read 20 bytes at a time from stream echoing back to stream
    loop {
        let mut read = [0; 1028];
        match stream.read(&mut read) {
            Ok(n) => {
                if n == 0 {
                    // connection was closed
                    break;
                }
                println!("{:?}", &read[0..n].to_owned());
            }
            Err(err) => {
                panic!("{err}");
            }
        }

        let mut buffer = String::new();
        match std::io::stdin().read_line(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }

                stream.write(buffer.as_bytes()).unwrap();
            }
            Err(err) => {
                panic!("{err}")
            }
        }
    }
}

fn main() {
    let port = args_os().nth(1).unwrap_or("8080".into());
    let port = port.to_str().unwrap();

    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();

    println!("echo listening on port: {port}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(_) => {
                println!("Error");
            }
        }
    }
}

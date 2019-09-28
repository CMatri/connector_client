use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::{thread, time};

fn main() {
    match TcpStream::connect("192.168.1.9:8080") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 8080");

            let msg = b"ping";

            stream.write(msg).unwrap();
            print!("Sent ping, awaiting reply...");
            let mut ping_data = [0 as u8; 4];
            match stream.read(&mut ping_data) {
                Ok(n) => {
                    if &ping_data == b"pong" {
                        println!("ponged");
                    } else {
                        let text = from_utf8(&ping_data).unwrap();
                        println!("unexpected reply: {}", text);
                    }
                },
                Err(e) => {
                    println!("failed to receive data: {}", e);
                }
            }

            let mut data = [0 as u8; 512];
            let sleep_time = time::Duration::from_millis(10);
            let sleep_time_err = time::Duration::from_millis(500);
            loop {
                match stream.read(&mut data) {
                    Ok(n) => {
                        println!("Received {} bytes:", n);
                        for d in data.iter() {
                            if *d != 0u8 {
                                print!("{}, ", d);
                            }
                        }
                        println!("");
                    },
                    Err(e) => {
                        println!("failed to receive data: {}", e);
                        thread::sleep(sleep_time_err);
                    }
                }
                thread::sleep(sleep_time);
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
use std::net::{TcpStream, UdpSocket};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::{thread, time};
use enigo::*;

const BUF_LEN: usize = 20;
const UDP_PORT: i32 = 5514;

fn socket(addr: &str) -> UdpSocket {
    UdpSocket::bind(format!("{}:{}", addr, UDP_PORT)).expect("Failed to open UDP socket.")
}

fn udp_listener() {
    let mut sock = socket("0.0.0.0");
    println!("Listening...");
    let mut buf = [0; 16];
    loop {
        match sock.recv_from(&mut buf) {
            Ok((amt, src)) => {
                if buf[0] == 0x01 && buf[1] == 0x02 && buf[2] == 0x03 {
                    println!("Got packet.");
                    let ip = src.ip().to_string();
                    let port = String::from_utf8(buf[3..7].iter().cloned().collect()).expect("fail").parse::<i32>().unwrap();
                    //sock.send_to(b"ABC", format!("{}:{}", ip, UDP_PORT)).expect("Failed to send UDP packet.");
                    thread::spawn(move || {
                        initiate_connection(ip, port);
                    });
                    //break;
                }
            },
            Err(e) => {
                println!("couldn't recieve a datagram: {}", e);
            }
        }
    }
}

fn packet_handler(enigo: &mut Enigo, bytes: &[u8; BUF_LEN]) {
    match bytes[0] {
        0x01 => { // Mouse
            let x = &bytes[1..7];
            let y = &bytes[8..13];
            let x_neg = bytes[13] == 0;
            let y_neg = bytes[14] == 0;
            let mut x_conv = String::from_utf8(x.iter().cloned().collect()).expect("fail").parse::<f32>().unwrap();
            let mut y_conv = String::from_utf8(y.iter().cloned().collect()).expect("fail").parse::<f32>().unwrap();
            if x_neg { x_conv *= -1.0; }
            if y_neg { y_conv *= -1.0; }
            enigo.mouse_move_relative((x_conv * 2.0) as i32, (y_conv * 3.0) as i32);
        }
        0x02 => {
            if bytes[1] == 0 {
                enigo.mouse_click(MouseButton::Left);
            } else {
                enigo.mouse_click(MouseButton::Right);    
            }
        } // Click,
        0x03 => {} // Volume,
        0x04 => {} // Key,
        _ => {}
    }
}

fn initiate_connection(ip: String, port: i32) {
    let mut enigo = Enigo::new();
    let mut i = 0;
    while i < 10 {
        match TcpStream::connect(format!("{}:{}", ip, port)) {
            Ok(mut stream) => {
                println!("Successfully connected to server in port 8080");

                let msg = b"ping";

                stream.write(msg).unwrap();
                print!("Sent ping, awaiting reply...");
                let mut ping_data = [0 as u8; 4];
                match stream.read(&mut ping_data) {
                    Ok(_) => {
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

                let mut data = [0 as u8; BUF_LEN];
                loop {
                    match stream.read(&mut data) {
                        Ok(_) => packet_handler(&mut enigo, &data),
                        Err(e) => {
                            println!("Failed to receive data: {}", e);
                            break;
                        }
                    }
                }
            },
            Err(e) => {
                println!("Failed to connect: {}", e);
                i += 1;
            }
        }
        println!("Terminated.");
    }
}

fn main() {
    udp_listener();
}
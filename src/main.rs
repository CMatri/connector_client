use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::{thread, time};
use enigo::*;

const BUF_LEN: usize = 20;

fn packet_handler(enigo: &mut Enigo, bytes: &[u8; BUF_LEN]) {
    match bytes[0] {
        0x01 => { // Mouse
            let x = &bytes[1..7];
            let y = &bytes[8..13];
            let x_neg = bytes[14] != 0;
            let y_neg = bytes[15] != 0;
            let mut x_conv = String::from_utf8(x.iter().cloned().collect()).expect("fail").parse::<f32>().unwrap();
            let mut y_conv = String::from_utf8(y.iter().cloned().collect()).expect("fail").parse::<f32>().unwrap();
            if x_neg { x_conv *= -1.0; }
            if y_neg { y_conv *= -1.0; }
            enigo.mouse_move_relative((x_conv * 3.0) as i32, (y_conv * 2.0) as i32);
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

fn main() {
    let mut enigo = Enigo::new();
    match TcpStream::connect("192.168.1.9:8080") {
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
        }
    }
    println!("Terminated.");
}
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::{thread, time};
use enigo::*;

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

            const buf_len: usize = 512;
            let mut data = [0 as u8; buf_len];
            let sleep_time = time::Duration::from_millis(10);
            let sleep_time_err = time::Duration::from_millis(500);
            loop {
                match stream.read(&mut data) {
                    Ok(n) => {
                        println!("Received {} bytes:", n);
                        let mut i = 0;
                        while i < buf_len {
                            let mut byte = data[i];
                            let mut x: Vec<Vec<u8>> = Vec::new();
                            let mut y: Vec<Vec<u8>> = Vec::new();
                            let mut temp_x: Vec<u8> = Vec::new();
                            let mut temp_y: Vec<u8> = Vec::new();
                            match byte { // welcome to my abomination. if you watch for a while it sprouts limbs and shrieks occasionally.
                                0u8 => break,
                                35u8 => { // #
                                    if data[i+1] == 35u8 && data[i+2] == 35u8 {
                                        i += 3;
                                        'outer: loop {
                                            byte = data[i];
                                            match byte {
                                                44u8 => { // ,
                                                    if data[i+1] == 44u8 && data[i+2] == 44u8 {
                                                        i += 3;
                                                        loop {
                                                            byte = data[i];
                                                            match byte {
                                                                59u8 => { // ;
                                                                    if data[i+1] == 59u8 && data[i+2] == 59u8 {
                                                                        x.push(temp_x.clone());
                                                                        y.push(temp_y.clone());
                                                                        let x_conv = String::from_utf8(temp_x).expect("fail").parse::<i32>().unwrap();
                                                                        let y_conv = String::from_utf8(temp_y).expect("fail").parse::<i32>().unwrap();
                                                                        println!("dX: {}", x_conv);
                                                                        println!("dY: {}", y_conv);
                                                                        enigo.mouse_move_relative(x_conv, y_conv);
                                                                        break 'outer;
                                                                    } else {
                                                                        i += 1;
                                                                    }
                                                                },
                                                                _ => {
                                                                    temp_y.push(byte);
                                                                    i += 1;
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        i += 1;
                                                    }
                                                },
                                                _ => {
                                                    temp_x.push(byte);
                                                    i += 1;
                                                }
                                            }
                                        }
                                    } else {
                                        i += 1;
                                    }
                                },
                                _ => i += 1
                            }

                        }
                        println!("");
                    },
                    Err(e) => {
                        println!("failed to receive data: {}", e);
                        break;
                        //thread::sleep(sleep_time_err);
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
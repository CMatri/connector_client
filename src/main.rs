use std::sync::mpsc;
use std::thread;
use std::net::{TcpStream, UdpSocket};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::time;
use enigo::*;
use winapi::um::mmeapi::{waveOutSetVolume, waveOutGetVolume};
use lerp::{Lerp, LerpIter};

const BUF_LEN: usize = 20;
const UDP_PORT: i32 = 5514;

fn udp_socket(addr: &str) -> UdpSocket {
    UdpSocket::bind(format!("{}:{}", addr, UDP_PORT)).expect("Failed to open UDP socket.")
}

fn udp_listener() {
    let sock = udp_socket("0.0.0.0");
    let mut buf = [0; 16];
    let sleep_time = time::Duration::from_millis(500);
    loop {
        println!("Listening...");
        match sock.recv_from(&mut buf) {
            Ok((_, src)) => {
                if buf[0] == 0x01 && buf[1] == 0x02 && buf[2] == 0x03 {
                    println!("Got broadcast packet.");
                    let ip = src.ip().to_string();
                    let port = String::from_utf8(buf[3..7].iter().cloned().collect()).expect("fail").parse::<i32>().unwrap();
                    //sock.send_to(b"ABC", format!("{}:{}", ip, UDP_PORT)).expect("Failed to send UDP packet.");
                    initiate_connection(ip, port);
                    //break;
                }
            },
            Err(e) => {
                println!("Couldn't receive a datagram: {}", e);
            }
        }

        thread::sleep(sleep_time);
    }
}

fn packet_handler(last_vol: &mut u32, enigo: &mut Enigo, bytes: &[u8; BUF_LEN], tx: &mpsc::Sender<(f64, f64)>) {
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
            tx.send(((x_conv * 2.0).into(), (y_conv * 4.0).into())).expect("Failed to send packet to mouse thread.");
            //enigo.mouse_move_relative((x_conv) as i32, (y_conv) as i32);
        }
        0x02 => { // Click,
            if bytes[1] == 0 {
                enigo.mouse_click(MouseButton::Left);
            } else {
                enigo.mouse_click(MouseButton::Right);    
            }
        }
        0x03 =>  { // Volume,
            unsafe {
                let mut vol: u32 = 0;
                waveOutGetVolume(std::ptr::null_mut(), &mut vol);
                if bytes[1] == 0 {
                    waveOutSetVolume(std::ptr::null_mut(), 0);//std::cmp::max(0, vol - 50));
                } else {
                    waveOutSetVolume(std::ptr::null_mut(), 0xFFFF);//std::cmp::min(0xFFFF, vol + 50));
                }
            }
        }
        0x04 => {} // Key,
        _ => {}
    }
}

fn mouse_thread(rx: mpsc::Receiver<(f64, f64)>) {
    thread::spawn(move || {
        let wait_time = time::Duration::from_millis(1);
        let mut enigo = Enigo::new();
        let mut cur_pos = (0f64, 0f64);
        /*let quad_bez = |x0: f64, x1: f64, x2: f64, steps: usize| -> Vec<f64> {
            (0..steps).map(|i| { 
                let x_0_1 = x0.lerp(x1, i as f64 / steps as f64);
                let x_1_2 = x1.lerp(x2, i as f64 / steps as f64);
                x_0_1.lerp(x_1_2, i as f64 / steps as f64)
            }).collect()
        };*/ // To use later
        let mut lerp = |p0: &(f64, f64), p1: &(f64, f64)| {
            if p0 == &(0f64, 0f64) { return; } // Prevents glitch on reception of first packet
            let vals_x: Vec<_> = p0.0.lerp_iter(p1.0, 5).collect();
            let vals_y: Vec<_> = p0.1.lerp_iter(p1.1, 5).collect();
            for (i, x) in vals_x.iter().enumerate() {
                enigo.mouse_move_relative(*x as i32, vals_y[i] as i32);
                thread::sleep(wait_time);
            }
        };
        loop { 
            match rx.recv() {
                Ok(pak) => { 
                    //enigo.mouse_move_relative((pak.0) as i32, (pak.1) as i32)
                    lerp(&cur_pos, &pak);
                    cur_pos = pak;
                }
                Err(_) => {}
            }
        }
    });
}

fn initiate_connection(ip: String, port: i32) {
    let mut last_vol = 0;
    let mut enigo = Enigo::new();
    let (tx, rx) = mpsc::channel();
    mouse_thread(rx);
    match TcpStream::connect(format!("{}:{}", ip, port)) {
        Ok(mut stream) => {
            println!("Successfully connected to {}:{}", ip, port);
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
                    Ok(_) => packet_handler(&mut last_vol, &mut enigo, &data, &tx),
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

fn main() {
    udp_listener();
}
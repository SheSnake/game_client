extern crate tokio;
use std::env;
use std::net::{ TcpStream };
use std::io::{ Write, Read };
use tokio::prelude::*;
extern crate bincode;
pub mod message;
use message::*;

fn login(stream: &mut TcpStream, user_id: &i64) {
    let id = *user_id as u8;
    let session = [id; 128];
    stream.write(&session).unwrap();
}

fn create_room(stream: &mut TcpStream, user_id: &i64) {
    let msg_type: u8 = 1;
    let length: i32 = 5 + 1 + 8 + 6;
    let op_type: u8 = 1;
    let room_id = [0u8; 6];
    stream.write(&msg_type.to_le_bytes()).unwrap();
    stream.write(&length.to_le_bytes()).unwrap();
    stream.write(&op_type.to_le_bytes()).unwrap();
    stream.write(&user_id.to_le_bytes()).unwrap();
    stream.write(&room_id).unwrap();
}

fn ready(stream: &mut TcpStream, user_id: &i64, room_id: &String) {
    let msg_type: u8 = 1;
    let length: i32 = 5 + 1 + 8 + 6;
    let op_type: u8 = 4;
    let room_id = room_id.clone().into_bytes();
    stream.write(&msg_type.to_le_bytes()).unwrap();
    stream.write(&length.to_le_bytes()).unwrap();
    stream.write(&op_type.to_le_bytes()).unwrap();
    stream.write(&user_id.to_le_bytes()).unwrap();
    stream.write(&room_id).unwrap();
}

fn join(stream: &mut TcpStream, user_id: &i64, room_id: &String) {
    let msg_type: u8 = 1;
    let length: i32 = 5 + 1 + 8 + 6;
    let op_type: u8 = 2;
    let room_id = room_id.clone().into_bytes();
    stream.write(&msg_type.to_le_bytes()).unwrap();
    stream.write(&length.to_le_bytes()).unwrap();
    stream.write(&op_type.to_le_bytes()).unwrap();
    stream.write(&user_id.to_le_bytes()).unwrap();
    stream.write(&room_id).unwrap();
}

fn read_msg(stream: &mut TcpStream) -> Vec<u8> {
    let mut buf = [0u8; 4096];
    let mut nread: usize = 0;
    while nread <= 5 {
        nread += stream.read(&mut buf[nread..]).unwrap()
    }
    let mut len_buf = [0u8; 4];
    for i in 0..4 {
        len_buf[i] = buf[i + 1];
    }
    let len = i32::from_le_bytes(len_buf) as usize;
    while nread <= len {
        nread += stream.read(&mut buf[nread..]).unwrap()
    }
    return buf.iter().cloned().collect();
    //let msg_type = buf[0];
    //match msg_type {
    //    1 => {
    //        let msg = bincode::deserialize::<RoomManageResult> (&buf).unwrap();
    //        let room_id: Vec<u8> = msg.room_id.iter().cloned().collect();
    //        let room_id = String::from_utf8(room_id).unwrap();
    //        println!("{}", room_id);
    //    },
    //    _ => {
    //    }
    //};
}

fn parse_to_room_manage_result(data: Vec<u8>) -> RoomManageResult {
    let msg = bincode::deserialize::<RoomManageResult> (&data).unwrap();
    return msg;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let user_id = args[1].parse::<i64>().unwrap();
    let mut room_id = "".to_string();
    if args.len() >= 3 {
        room_id = args[2].clone();
    }

    let mut stream = TcpStream::connect("0.0.0.0:8890");
    if let Ok(mut stream) = stream {
        login(&mut stream, &user_id);
        if room_id != "" {
            join(&mut stream, &user_id, &room_id);
        } else {
            create_room(&mut stream, &user_id);
            let mut msg = parse_to_room_manage_result(read_msg(&mut stream));
            room_id = String::from_utf8(msg.room_id.clone()).unwrap();
        }
        println!("create room: {}", room_id);
        ready(&mut stream, &user_id, &room_id);
        loop {}
    }
    loop {}
}

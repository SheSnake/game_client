extern crate tokio;
use std::mem;
use std::env;
use std::net::{ TcpStream };
use std::io::{ Write, Read };
use tokio::prelude::*;
extern crate bincode;
pub mod message;
use message::*;
use message::action::*;

fn login(stream: &mut TcpStream, user_id: &i64) {
    let id = *user_id as u8;
    let session = [id; 128];
    stream.write(&session).unwrap();
}

fn create_room(stream: &mut TcpStream, user_id: &i64) {
    let msg_type: i8 = unsafe{ mem::transmute(MsgType::RoomOp) };
    let length: i32 = 5 + 1 + 8 + 6;
    let op_type: i8 = unsafe{ mem::transmute(OpType::CreateRoom)};
    let room_id = [0u8; 6];
    stream.write(&msg_type.to_le_bytes()).unwrap();
    stream.write(&length.to_le_bytes()).unwrap();
    stream.write(&op_type.to_le_bytes()).unwrap();
    stream.write(&user_id.to_le_bytes()).unwrap();
    stream.write(&room_id).unwrap();
}

fn ready(stream: &mut TcpStream, user_id: &i64, room_id: &String) {
    let msg_type: i8 = unsafe{ mem::transmute(MsgType::RoomOp) };
    let length: i32 = 5 + 1 + 8 + 6;
    let op_type: u8 = unsafe{ mem::transmute(OpType::ReadyRoom)};
    let room_id = room_id.clone().into_bytes();
    stream.write(&msg_type.to_le_bytes()).unwrap();
    stream.write(&length.to_le_bytes()).unwrap();
    stream.write(&op_type.to_le_bytes()).unwrap();
    stream.write(&user_id.to_le_bytes()).unwrap();
    stream.write(&room_id).unwrap();
}

fn join(stream: &mut TcpStream, user_id: &i64, room_id: &String) {
    let msg_type: i8 = unsafe{ mem::transmute(MsgType::RoomOp) };
    let length: i32 = 5 + 1 + 8 + 6;
    let op_type: i8 = unsafe{ mem::transmute(OpType::JoinRoom)};
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
    while nread < HEADER_SIZE {
        nread += stream.read(&mut buf[nread..HEADER_SIZE]).unwrap()
    }
    let mut len_buf = [0u8; 4];
    for i in 0..4 {
        len_buf[i] = buf[i + 1];
    }
    let len = i32::from_le_bytes(len_buf) as usize;
    while nread < len {
        nread += stream.read(&mut buf[nread..len]).unwrap()
    }
    return buf.iter().cloned().collect();
}

fn parse_to_room_manage_result(data: Vec<u8>) -> RoomManageResult {
    let msg = bincode::deserialize::<RoomManageResult> (&data).unwrap();
    return msg;
}

fn parse_header(data: Vec<u8>) -> Header {
    let header = bincode::deserialize::<Header> (&data).unwrap();
    return header;
}

fn parse_to_room_update(data: Vec<u8>) -> RoomUpdate {
    let msg = bincode::deserialize::<RoomUpdate> (&data).unwrap();
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
        let mut msg = read_msg(&mut stream);
        let msg = bincode::deserialize::<AuthenResult> (&msg).unwrap();
        match unsafe{ mem::transmute(msg.code) } {
            Code::AuthenOk => {
                println!("auth {} success", user_id);
            },
            _ => {
                println!("auth {} fail", msg.code);
                return;
            }
        };
        if room_id != "" {
            join(&mut stream, &user_id, &room_id);
            ready(&mut stream, &user_id, &room_id);
        } else {
            create_room(&mut stream, &user_id);
        }
        let mut start = false;
        while !start {
            let msg = read_msg(&mut stream);
            let header = parse_header(msg.clone());
            match unsafe { mem::transmute(header.msg_type) } {
                MsgType::RoomManageResult => {
                    let result = bincode::deserialize::<RoomManageResult>(&msg).unwrap();
                    let room_id = String::from_utf8(result.room_id.clone()).unwrap();
                    println!("user_id:{}, in room_id:{}, op_type:{}", user_id, room_id, result.op_type);
                    match unsafe{ mem::transmute(result.op_type) } {
                        OpType::CreateRoom => {
                            ready(&mut stream, &user_id, &room_id);
                        },
                        _ => {},
                    }
                },
                MsgType::RoomSnapshot => {
                    let snapshot = bincode::deserialize::<RoomSnapshot>(&msg).unwrap();
                    let room_id = String::from_utf8(snapshot.room_id.clone()).unwrap();
                    println!("room_id:{} status: user_pos:{:?} user_ready_status:{:?}",
                        room_id, snapshot.user_pos, snapshot.user_ready_status);
                },
                MsgType::RoomUpdate => {
                    start = true;
                },
                MsgType::GameSnapshot => {
                    start = true;
                }
                _ => {
                    println!("read msg type:{}, len:{}", header.msg_type, header.len);
                }
            }
        }
        println!("game start!");
        let mut my_cards: Vec<u8> = Vec::new();
        loop {
            let mut msg = read_msg(&mut stream);
            let header = parse_header(msg.clone());
            match unsafe { mem::transmute(header.msg_type) } {
                MsgType::GameUpdate => {
                    let update = bincode::deserialize::<GameUpdate> (&msg).unwrap();
                    match unsafe { mem::transmute(update.op_type) } {
                        Action::DealBeginCard => {
                            println!("recv begin card: {:?}", update.provide_cards);
                            my_cards = update.provide_cards;
                        },
                        Action::DealNextCard => {
                            println!("cur_round:{} cur_step:{} user: {} on_hand:{:?} recv new card: {:?}", update.game_info.cur_game_round, update.game_info.cur_game_step, update.game_info.user_id, update.provide_cards, update.target);
                            my_cards.push(update.target);
                            my_cards.sort();
                        },
                        Action::Pop => {
                            if update.game_info.user_id == user_id {
                                let mut target: usize = 0;
                                let mut found = false;
                                for (ix, &card) in my_cards.iter().enumerate() {
                                    if card == update.target {
                                        target = ix;
                                        found = true;
                                        break;
                                    }
                                }
                                if found: {
                                    my_cards.remove(target);
                                }
                            }
                            println!("cur_round:{} cur_step:{} user: {}, do {}, target:{}", update.game_info.cur_game_round, update.game_info.cur_game_step, update.game_info.user_id, update.op_type, update.target);
                        },
                        Action::Peng => {
                            println!("cur_round:{} cur_step:{} user: {}, do {}, target:{}", update.game_info.cur_game_round, update.game_info.cur_game_step,  update.game_info.user_id, update.op_type, update.target);
                        },
                        Action::Chi => {
                            println!("cur_round:{} cur_step:{} user: {}, do {}, target:{}", update.game_info.cur_game_round, update.game_info.cur_game_step,  update.game_info.user_id, update.op_type, update.target);
                        },
                        Action::Gang => {
                            println!("cur_round:{} cur_step:{} user: {}, do {}, target:{}", update.game_info.cur_game_round, update.game_info.cur_game_step,  update.game_info.user_id, update.op_type, update.target);
                        },
                        Action::Hu => {
                            println!("cur_round:{} cur_step:{} user: {}, win by {}, target:{}", update.game_info.cur_game_round, update.game_info.cur_game_step,  update.game_info.user_id, update.op_type, update.target);
                        },
                        _ => {}
                    }
                },
                MsgType::GameOpPack => {
                    let op_list = bincode::deserialize::<GameOperationPack> (&msg).unwrap();
                    let mut cur_info = format!("cur_round:{} cur_step:{} ", op_list.operations[0].game_info.cur_game_round, op_list.operations[0].game_info.cur_game_step);
                    let mut choose = "".to_string();
                    for op in op_list.operations.iter() {
                        choose += &format!("do {}, target {}, ", op.op_type, op.target);
                    }
                    println!("{} can choose to: {}", cur_info, choose);
                    let data: &[u8] = &bincode::serialize::<GameOperation>(&op_list.operations[0]).unwrap();
                    stream.write(data);
                },
                MsgType::GameRoundUpdate => {
                    let update = bincode::deserialize::<GameRoundUpdate> (&msg).unwrap();
                    match unsafe { mem::transmute(update.round_info_type) } {
                        RoundInfoType::RoundStart => {
                            let mut content = format!("round {} start:", update.cur_round).to_string();
                            for i in 0..4 {
                                content += &format!(" user:{} score:{}", i, update.user_cur_score[i]);
                            }
                            println!("{}", content);
                        },
                        RoundInfoType::RoundOver => {
                            let mut content = format!("round {} over:", update.cur_round).to_string();
                            for i in 0..4 {
                                content += &format!(" user:{} score:{}", i, update.user_cur_score[i]);
                            }
                            println!("{}", content);
                        }
                    }
                    my_cards.clear();
                }
                _ => {
                }
            };
        }
    }
    loop {}
}

use actix::{Actor, StreamHandler, AsyncContext, Handler, Addr};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::collections::HashMap;
use std::sync::{Mutex, PoisonError, MutexGuard};
use crate::server::error;
use libflate::gzip::Encoder;
use std::io::{self};
use std::collections::hash_map::RandomState;
use serde::{Serialize, Deserialize};
use actix_web::http::HeaderValue;
use actix_web::http::header::ToStrError;
use std::borrow::Borrow;
use std::cell::RefCell;
use crate::server::error::AError;
use chrono::{DateTime, Local};


mod date_format {
    use std::error::Error;

    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Deserializer, Serializer};
    use serde::de::{Unexpected, Visitor};
    use serde::__private::Formatter;

    pub fn serialize<S>(
        date: &DateTime<Local>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(date.format("%Y-%m-%d %H:%M:%S").to_string().as_str())//序列化直接转为时间戳
    }


    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Local>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let timestamp = deserializer.deserialize_any(I64)?;//定义了I64访问者来处理格式转换
        Ok(Local.timestamp_millis(timestamp))
    }

    struct I64;

    impl<'de> Visitor<'de> for I64 {
        type Value = i64;
        fn expecting<'a>(&self, formatter: &mut Formatter<'a>) -> std::fmt::Result {
            write!(formatter, "is an integer")
        }

        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E> where
            E: Error, {//这个方法在这个业务里没有实现必要，只是为了例子解释
            println!("v {}", v);
            Ok(v as i64)
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> where
            E: Error, {
            println!("v {}", v);
            Ok(v)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where
            E: Error, {//这个例子如果不实现这个方法，就会报错
            println!("v {}", v);
            Ok(v as i64)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SocketSession {
    uri: String,
    agent: String,
    ip: String,
    #[serde(with = "date_format")]
    create_time: DateTime<Local>,
}

#[macro_use]
lazy_static::lazy_static! {
    static ref USER_LIST :Mutex<HashMap<String, (SocketSession, Addr<DefaultSocket>)>> = {
       Mutex::new(HashMap::new())
    };
}

#[derive(Debug)]
struct DefaultSocket {
    uri: String,
    agent: String,
    ip: String,
    time: DateTime<Local>,
}


impl DefaultSocket {
    fn get_user_info(&self) -> Result<(&str, &str, &str), error::AError> {
        let urls: Vec<&str> = self.uri.split("/").collect();
        if urls.len() <= 4 {
            return std::result::Result::Err(error::AError::new("uri error"));
        }
        let user_id = urls[2];
        let token = urls[3];
        let code = urls[4];
        return std::result::Result::Ok((user_id, token, code));
    }
    fn get_socket(&self) -> SocketSession {
        SocketSession {
            uri: self.uri.to_owned(),
            agent: self.agent.to_owned(),
            ip: self.ip.to_owned(),
            create_time: self.time,
        }
    }
}


impl Actor for DefaultSocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for DefaultSocket {
    fn handle(&mut self, message: Result<ws::Message, ws::ProtocolError>, context: &mut Self::Context) {
        match message {
            Ok(ws::Message::Ping(message)) => context.pong(&message),
            Ok(ws::Message::Text(_)) => {
                match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                    Ok(value) => {
                        context.text(format!("{}", value.as_millis()));
                    }
                    Err(_) => {
                        context.text(format!("{}", "0"));
                    }
                }
            }
            Ok(ws::Message::Binary(data)) => context.binary(data),
            _ => (),
        }
    }

    fn started(&mut self, context: &mut Self::Context) {
        context.text("Welcome To WebSocket Service");

        let mut info = match self.get_user_info() {
            Ok(value) => { value }
            Err(_) => {
                context.text("token error");
                context.close(Option::None);
                return;
            }
        };
        match (*USER_LIST).lock() {
            Ok(mut value) => {
                log::info!("[Socket Add] user: {}, token:{}, code:{}", info.0, info.1, info.2);
                value.insert(String::from(info.1), (self.get_socket(), context.address()));
            }
            Err(_) => {}
        }
    }


    fn finished(&mut self, _: &mut Self::Context) {
        let info = match self.get_user_info() {
            Ok(value) => { value }
            Err(_) => {
                return;
            }
        };
        match (*USER_LIST).lock() {
            Ok(mut value) => {
                log::info!("[Socket Remove] user: {}, token:{}, code:{}", info.0, info.1, info.2);
                value.remove(info.0);
                value.remove(info.1);
            }
            Err(_) => {}
        };
    }
}

pub async fn ws(request: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    // /ws/用户ID/用户token/随机代码
    let ip = request.peer_addr().unwrap().ip().to_string();
    let uri = request.uri().to_string();
    let agent = match request.headers().get("User-Agent") {
        None => { "" }
        Some(v) => {
            match v.to_str() {
                Ok(v) => { v }
                Err(_) => { "" }
            }
        }
    }.to_owned();
    ws::start(DefaultSocket { uri, ip, agent, time: Local::now() }, &request, stream)
}

struct SocketMessage {
    message: String
}

impl actix::Message for SocketMessage { type Result = (); }

impl Handler<SocketMessage> for DefaultSocket {
    type Result = ();

    fn handle(&mut self, msg: SocketMessage, ctx: &mut Self::Context) {
        let mut encoder = match Encoder::new(Vec::new()) {
            Ok(value) => {
                value
            }
            Err(_) => {
                return;
            }
        };
        match io::copy(&mut msg.message.as_bytes(), &mut encoder) {
            Ok(_) => {}
            Err(_) => {
                return;
            }
        }
        let encoded_data = match encoder.finish().into_result() {
            Ok(value) => {
                value
            }
            Err(_) => {
                return;
            }
        };
        ctx.binary(encoded_data);
    }
}

pub fn send(id: &str, message: String) -> Result<bool, AError> {
    let result = match (*USER_LIST).lock() {
        Ok(value) => {
            value
        }
        Err(_) => {
            return Err(AError::service("读取用户列表失败"));
        }
    };
    if result.contains_key(id) {
        result[id].1.do_send(SocketMessage { message });
        Ok(true)
    } else {
        Err(AError::service("用户不在线"))
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct SocketUser {
    token: String,
    ip: String,
}

pub fn get_list() -> Vec<SocketSession> {
    let user_list = match (*USER_LIST).lock() {
        Ok(value) => {
            value
        }
        Err(_) => { return vec![]; }
    };
    let mut result_list = vec![];
    for key in user_list.keys() {
        let item = &user_list[key].0;
        result_list.push(item.clone());
    }
    return result_list;
}
use actix_web::{Responder, HttpResponse, HttpRequest, web, Error};
use actix_multipart::Multipart;
use crate::{server, data};
use actix_web::web::Payload;
use actix_web::body::Body;
use serde_json::Value;
use std::collections::HashMap;
use crate::server::error::{AError, XError};

pub async fn send(mut payload: Payload) -> impl Responder {
    server::tool::handle((move || async move {
        let message: Value = server::tool::get_body_json(payload).await?;
        let to = match message.get("to") {
            None => {
                return Err(XError::AError(AError::p()));
            }
            Some(value) => {
                match value.as_str() {
                    None => {
                        return Err(XError::AError(AError::p()));
                    }
                    Some(value) => { value }
                }
            }
        };
        let value = match message.get("value") {
            None => {
                return Err(XError::AError(AError::p()));
            }
            Some(value) => {
                match value.as_str() {
                    None => {
                        return Err(XError::AError(AError::p()));
                    }
                    Some(value) => { value.to_owned() }
                }
            }
        };
        server::socket::send(&to, value)?;
        Ok(Value::Bool(true))
    })().await)
}

pub async fn get_list() -> impl Responder {
    server::tool::ApiResponse::success(&server::socket::get_list())
}


// let d = data::test::TestConfig::default();
// Multipart
//let mut hash = server::tool::get_form_data(payload).await;

// serde_json::from_str().unwrap();
//server::socket::send(&args["id"], args["data"].to_owned());
// request.b

//chrono用法样例
//时间
//动态序列化
//返回所有权
//自定义序列化

// let r = hash.get("ddd").unwrap().to_owned();
// let r = String::from_utf8(hash.get("123").unwrap().to_vec().to_owned()).unwrap();
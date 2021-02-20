use std::collections::HashMap;
use actix_web::{HttpRequest, web, HttpResponse, ResponseError, Responder};
use actix_web::web::{Payload, BytesMut, Bytes};
use futures_core::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::RandomState;
use actix_multipart::{Multipart, Field, MultipartError};
use futures::{StreamExt, TryStreamExt, Future};
use actix_web::body::Body;
use serde_json::Value;
use crate::server::error::{AError, XError};
use actix_web::error::PayloadError;
use std::error::Error;
use std::str::Utf8Error;
use actix_web::http::header::ContentDisposition;
use async_trait::async_trait;

///限制Body 大小
const MAX_SIZE: usize = 262_144;

/// 读取查询参数
pub async fn get_query_args(request: &HttpRequest) -> HashMap<String, String> {
    parsing(request.query_string())
}

/// 读取内容的Json
pub async fn get_body_json<T>(mut payload: Payload) -> Result<T, XError> where T: for<'a> Deserialize<'a> {
    let mut body = BytesMut::new();
    let item = read_body(payload).await?;
    body.extend_from_slice(&item);
    let json = std::str::from_utf8(&body)?;
    Ok(serde_json::from_str(json)?)
}

/// 读取内容的urlencoded
pub async fn get_form_urlencoded(mut payload: Payload) -> Result<HashMap<String, String>, XError> {
    let mut body = BytesMut::new();
    let item = read_body(payload).await?;
    body.extend_from_slice(&item);
    let url = std::str::from_utf8(&body)?;
    Ok(parsing(url))
}

/// 读取表单
pub async fn get_form_data(mut payload: Multipart) -> Result<HashMap<String, Bytes>, XError> {
    let mut body = HashMap::new();
    loop {
        let mut field = match payload.try_next().await? {
            None => { break; }
            Some(value) => { value }
        };
        let content_type = match field.content_disposition() {
            None => { break; }
            Some(value) => { value }
        };
        body.insert(match content_type.get_name() {
            None => { break; }
            Some(value) => { value.to_owned() }
        }, match field.next().await {
            None => { break; }
            Some(value) => { value? }
        });
    }
    Ok(body)
}

/// 读取内容 二进制
pub async fn get_byte(mut payload: Payload) -> Result<Vec<u8>, XError> {
    let mut body = BytesMut::new();
    let item = read_body(payload).await?;
    body.extend_from_slice(&item);
    Ok(body.to_vec())
}

fn parsing(url: &str) -> HashMap<String, String> {
    let mut args: HashMap<String, String> = HashMap::new();
    url.split("&").for_each(|it| {
        let items: Vec<&str> = it.split(" = ").collect();
        let value = match urlencoding::decode(items[1]) {
            Ok(value) => { value.to_owned() }
            Err(_) => {
                String::from(";")
            }
        };
        args.insert(items[0].to_owned(), value);
    });
    args
}

async fn read_body(mut payload: Payload) -> Result<Bytes, AError> {
    Ok(match payload.next().await {
        None => {
            return Err(AError::parameter("读取内容失败"));
        }
        Some(value) => {
            match value {
                Ok(value) => { value }
                Err(_) => {
                    return Err(AError::parameter("读取内容失败"));
                }
            }
        }
    })
}


pub fn handle<T>(f: Result<T, XError>) -> HttpResponse
    where T: Sized + Serialize {
    return match f {
        Ok(value) => {
            ApiResponse::success(&value)
        }
        Err(error) => {
            ApiResponse::error("系统异常")
        }
    };
}

/// 响应对象
pub struct ApiResponse;

impl ApiResponse {
    /// 响应成功
    pub fn success<T>(data: &T) -> HttpResponse where T: Sized + Serialize {
        let response_json = serde_json::to_string(data).unwrap();
        HttpResponse::Ok()
            .set_header("Content-Type", "application/json;charset = utf-8")
            .body(format!("
                {}\"data\":{},\"code\":\"{}\"{}",
                          "{", response_json, "200", "}"))
    }

    /// 响应失败
    pub fn error(message: &str) -> HttpResponse {
        HttpResponse::Ok()
            .set_header("Content-Type", "application/json;charset=utf-8")
            .body(format!("{}\"message\":\"{}\",\"code\":\"{}\"{}",
                          "{", message, "500", "}"))
    }
}
use std::fmt;
use std::net::Ipv4Addr;

use crate::curl;
use crate::result::MtcapError;

pub struct Gateway {
    ip: Ipv4Addr,
    username: String,
    password: String,
}

impl Gateway {
    pub fn new(ip: Ipv4Addr, username: String, password: String) -> Self {
        Self {
            ip,
            username,
            password,
        }
    }
}

pub struct Token {
    ip: Ipv4Addr,
    token: String,
}

impl Token {
    fn new(ip: Ipv4Addr, token: String) -> Self {
        Self { ip, token }
    }
}

pub fn login(gateway: &Gateway) -> Result<Token, MtcapError> {
    let response = curl::get(format!(
        "https://{}/api/login?username={}&password={}",
        gateway.ip, gateway.username, gateway.password
    ))?;

    let json = json::parse(&response)?;
    let token_string = json["result"]["token"].to_string();

    let token = Token::new(gateway.ip, token_string);

    Ok(token)
}

pub fn save_apply(token: &Token) -> Result<(), MtcapError> {
    curl::post(get_url(token, "command/save_apply"))?;

    Ok(())
}

pub fn logout(token: &Token) -> Result<(), MtcapError> {
    curl::get(get_url(token, "logout"))?;

    Ok(())
}

pub fn get_url<T: fmt::Display>(token: &Token, api: T) -> String {
    format!("https://{}/api/{api}?token={}", token.ip, token.token)
}

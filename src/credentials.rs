use std::fmt;

use anyhow::Result;

use crate::curl;

const IPV4_LENGTH: usize = 4;

#[derive(Clone)]
struct Ipv4 {
    digits: [u8; IPV4_LENGTH],
}

impl fmt::Display for Ipv4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.digits[0], self.digits[1], self.digits[2], self.digits[3],
        )
    }
}

impl Ipv4 {
    fn new(ip: [u8; IPV4_LENGTH]) -> Self {
        Self { digits: ip }
    }
}

pub struct Gateway {
    ip: Ipv4,
    username: String,
    password: String,
}

impl Gateway {
    pub fn new(ip: [u8; IPV4_LENGTH], username: String, password: String) -> Self {
        Self {
            ip: Ipv4::new(ip),
            username,
            password,
        }
    }
}

pub struct Token {
    ip: Ipv4,
    token: String,
}

impl Token {
    fn new(ip: Ipv4, token: String) -> Self {
        Self { ip, token }
    }
}

pub fn login(gateway: &Gateway) -> Result<Token> {
    let response = curl::get(format!(
        "https://{}/api/login?username={}&password={}",
        gateway.ip, gateway.username, gateway.password
    ))?;

    let json = json::parse(&response)?;
    let token_string = json["result"]["token"].to_string();

    let token = Token::new(gateway.ip.clone(), token_string);

    Ok(token)
}

pub fn save_apply(token: &Token) -> Result<()> {
    curl::post(get_url(token, "command/save_apply"))?;

    Ok(())
}

pub fn logout(token: &Token) -> Result<()> {
    curl::get(get_url(token, "logout"))?;

    Ok(())
}

pub fn get_url(token: &Token, api: &str) -> String {
    format!("https://{}/api/{}?token={}", token.ip, api, token.token)
}

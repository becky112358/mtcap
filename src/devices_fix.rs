use std::error::Error;
use std::net::IpAddr;
use std::os::windows::process::CommandExt as _;
use std::process::Command;
use std::process::Output;

use serde::{de, Deserialize};

use crate::curl::CREATE_NO_WINDOW;

#[derive(Debug, Deserialize)]
struct MultitechApiLoraDevices {
    result: Vec<Device>,
}

#[derive(Debug, Deserialize)]
struct Device {
    deveui: String,
}

pub fn remove_all_devices_one_by_one(ip: &IpAddr, token: &str) -> Result<(), Box<dyn Error>> {
    let token = Some(token);
    let url = format!("https://{ip}/api/lora/devices");
    let response: MultitechApiLoraDevices = get(&url, token)?;
    for device in response.result {
        let url = format!("https://{ip}/api/lora/devices/{}", device.deveui);
        delete(url, token)?;
        println!("removed device {device:?}");
    }

    Ok(())
}

pub fn get<T: de::DeserializeOwned>(url: &str, token: Option<&str>) -> Result<T, Box<dyn Error>> {
    let output = Command::new("curl")
        .arg("-k")
        .arg(url)
        .token(token)
        .creation_flags(CREATE_NO_WINDOW)
        .output()?;

    Ok(serde_json::from_slice(&output.stdout)?)
}

pub fn delete(url: String, token: Option<&str>) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("curl")
        .arg("-k")
        .arg(url)
        .arg("-X")
        .arg("DELETE")
        .token(token)
        .creation_flags(CREATE_NO_WINDOW)
        .output()?)
}

trait WithToken {
    fn token<'a>(&'a mut self, token: Option<&str>) -> &'a mut Self;
}

impl WithToken for Command {
    fn token<'a>(&'a mut self, token: Option<&str>) -> &'a mut Self {
        if let Some(token) = token.map(|token| format!("token={token}")) {
            self.arg("--cookie").arg(token)
        } else {
            self
        }
    }
}

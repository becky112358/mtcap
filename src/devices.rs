use std::fmt;
use std::str::FromStr;

use anyhow::{Error, Result};

use strum_macros::Display;

use crate::credentials::{get_url, save_apply, Token};
use crate::curl;

const EUI_LENGTH: usize = 8;

const KEY_LENGTH: usize = 16;

pub struct Device {
    device_eui: Eui,
    join_eui: Eui,
    application_key: Key,
    class: Class,
    device_profile: DeviceProfile,
    network_profile: Class,
}

impl Device {
    pub fn new(
        device_eui: Eui,
        join_eui: Eui,
        application_key: Key,
        class: Class,
        device_profile: DeviceProfile,
        network_profile: Class,
    ) -> Self {
        Self {
            device_eui,
            join_eui,
            application_key,
            class,
            device_profile,
            network_profile,
        }
    }
}

pub struct Eui {
    digits: [u8; EUI_LENGTH],
}

impl fmt::Display for Eui {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}",
            self.digits[0],
            self.digits[1],
            self.digits[2],
            self.digits[3],
            self.digits[4],
            self.digits[5],
            self.digits[6],
            self.digits[7],
        )
    }
}

impl FromStr for Eui {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let eui_vec = string_to_vec_u8(input, EUI_LENGTH, Some('-'))?;
        Ok(Eui::new(eui_vec.try_into().unwrap()))
    }
}

impl Eui {
    pub fn new(eui: [u8; EUI_LENGTH]) -> Self {
        Self { digits: eui }
    }
}

pub struct Key {
    digits: [u8; KEY_LENGTH],
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} \
             {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            self.digits[0],
            self.digits[1],
            self.digits[2],
            self.digits[3],
            self.digits[4],
            self.digits[5],
            self.digits[6],
            self.digits[7],
            self.digits[8],
            self.digits[9],
            self.digits[10],
            self.digits[11],
            self.digits[12],
            self.digits[13],
            self.digits[14],
            self.digits[15],
        )
    }
}

impl FromStr for Key {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let key_vec = string_to_vec_u8(input, KEY_LENGTH, None)?;
        Ok(Key::new(key_vec.try_into().unwrap()))
    }
}

impl Key {
    pub fn new(key: [u8; KEY_LENGTH]) -> Self {
        Self { digits: key }
    }
}

pub fn string_to_vec_u8(
    input: &str,
    output_length: usize,
    padding_character: Option<char>,
) -> Result<Vec<u8>> {
    if input.len() == output_length * 2 + output_length - 1 {
        let mut padding_char = padding_character;
        let mut input_unpadded = String::new();
        for (i, c) in input.chars().enumerate() {
            if (i + 1) % 3 == 0 {
                if let Some(pc) = padding_char {
                    if c != pc {
                        return Err(anyhow::anyhow!("{} has inconsistent passing", input));
                    }
                } else {
                    padding_char = Some(c);
                }
            } else {
                input_unpadded.push(c);
            }
        }
        string_unpadded_to_vec_u8(&input_unpadded, output_length)
    } else {
        string_unpadded_to_vec_u8(input, output_length)
    }
}

fn string_unpadded_to_vec_u8(input: &str, output_length: usize) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(output_length);

    if input.len() == output_length * 2 {
        for (i, _) in input.chars().enumerate() {
            if i % 2 == 0 {
                let digits = u8::from_str_radix(&input[i..=i + 1], 16)?;
                output.push(digits);
            } else {
                continue;
            }
        }
        Ok(output)
    } else {
        Err(anyhow::anyhow!(
            "{} is not of length {}",
            input,
            output_length * 2
        ))
    }
}

#[derive(Display)]
pub enum Class {
    A,
    B,
    C,
}

#[derive(Display)]
pub enum DeviceProfile {
    #[strum(serialize = "AS923")]
    As923,
    #[strum(serialize = "AU915")]
    Au915,
    #[strum(serialize = "EU868")]
    Eu868,
    #[strum(serialize = "IN865")]
    In865,
    #[strum(serialize = "KR920")]
    Kr920,
    #[strum(serialize = "US915")]
    Us915,
}

pub fn enable(token: &Token, devices: &[Device]) -> Result<()> {
    let devices_json = create_json(devices)?;

    curl::put(get_url(token, "loraNetwork/whitelist"), devices_json)?;

    save_apply(token)?;

    Ok(())
}

fn create_json(devices: &[Device]) -> Result<json::JsonValue> {
    let mut devices_json = json::object! {
        devices: [],
        enabled: true,
    };
    for device in devices.iter() {
        let mut application_key = device.application_key.to_string();
        application_key.retain(|c| c.is_digit(16));
        let device_json = json::object! {
            appeui: device.join_eui.to_string(),
            appkey: application_key,
            class: device.class.to_string(),
            deveui: device.device_eui.to_string(),
            device_profile_id: format!("LW102-OTA-{}", device.device_profile),
            network_profile_id: format!("DEFAULT-CLASS-{}", device.network_profile),
        };
        devices_json["devices"].push(device_json)?;
    }

    Ok(devices_json)
}

#[rustfmt::skip]
#[cfg(test)]
#[path = "./test_devices.rs"]
mod test_devices;

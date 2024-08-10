use std::fmt;
use std::io::{self, Error, ErrorKind};
use std::str::FromStr;

use chrono;
use strum_macros::Display;

use crate::credentials::{get_url, save_apply, Token};
use crate::curl;
use crate::result::MtcapError;

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
    pub const fn new(
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

#[derive(Clone, Debug, PartialEq)]
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

    fn from_str(input: &str) -> io::Result<Self> {
        let eui_vec = string_to_vec_u8(input, EUI_LENGTH, Some('-'))?;
        Ok(Eui::new(eui_vec.try_into().unwrap()))
    }
}

impl Eui {
    pub const fn new(eui: [u8; EUI_LENGTH]) -> Self {
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

    fn from_str(input: &str) -> io::Result<Self> {
        let key_vec = string_to_vec_u8(input, KEY_LENGTH, None)?;
        Ok(Key::new(key_vec.try_into().unwrap()))
    }
}

impl Key {
    pub const fn new(key: [u8; KEY_LENGTH]) -> Self {
        Self { digits: key }
    }

    fn to_string_no_spaces(&self) -> String {
        let mut output = self.to_string();
        output.retain(|c| c.is_ascii_hexdigit());
        output
    }
}

pub fn string_to_vec_u8(
    input: &str,
    output_length: usize,
    padding_character: Option<char>,
) -> io::Result<Vec<u8>> {
    if input.len() == output_length * 2 + output_length - 1 {
        let mut padding_char = padding_character;
        let mut input_unpadded = String::new();
        for (i, c) in input.chars().enumerate() {
            if (i + 1) % 3 == 0 {
                if let Some(pc) = padding_char {
                    if c != pc {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            format!("{} has inconsistent padding", input),
                        ));
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

fn string_unpadded_to_vec_u8(input: &str, output_length: usize) -> io::Result<Vec<u8>> {
    let mut output = Vec::with_capacity(output_length);

    if input.len() == output_length * 2 {
        for (i, _) in input.chars().enumerate() {
            if i % 2 == 0 {
                let digits = match u8::from_str_radix(&input[i..=i + 1], 16) {
                    Ok(d) => d,
                    Err(e) => return Err(Error::new(ErrorKind::InvalidInput, format!("{:?}", e))),
                };
                output.push(digits);
            } else {
                continue;
            }
        }
        Ok(output)
    } else {
        Err(Error::new(
            ErrorKind::InvalidInput,
            format!("{} is not of length {}", input, output_length * 2),
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

pub fn get_count(token: &Token) -> Result<usize, MtcapError> {
    let gateway_response = curl::get(get_url(token, "loraNetwork/whitelist"))?;
    let devices_json = &json::parse(&gateway_response)?["result"]["devices"];

    Ok(devices_json.len())
}

pub fn enable(token: &Token, devices: &[Device]) -> Result<(), MtcapError> {
    let mut devices_json = json::object! {
        devices: [],
        enabled: true,
    };
    for device in devices.iter() {
        devices_json["devices"].push(create_json(device))?;
    }

    curl::put(get_url(token, "loraNetwork/whitelist"), devices_json)?;

    save_apply(token)?;

    Ok(())
}

pub fn clear(token: &Token) -> Result<(), MtcapError> {
    enable(token, &[])?;

    let gateway_response = curl::get(get_url(token, "lora/devices"))?;
    let devices_json = json::parse(&gateway_response)?["result"].clone();
    let mut index = 0;
    while !devices_json[index].is_null() {
        let device_eui = devices_json[index]["deveui"].to_string();
        curl::delete(get_url(token, format!("lora/devices/{device_eui}")))?;

        index += 1;
    }

    save_apply(token)?;

    Ok(())
}

pub fn add(token: &Token, devices: &[Device]) -> Result<(), MtcapError> {
    let gateway_response = curl::get(get_url(token, "loraNetwork/whitelist"))?;
    let mut devices_json = json::parse(&gateway_response)?["result"].clone();

    for device_wanted in devices {
        let mut included = false;
        let mut index = 0;
        while !devices_json["devices"][index].is_null() {
            let device_eui_existing =
                Eui::from_str(&devices_json["devices"][index]["deveui"].to_string())?;
            if device_eui_existing.eq(&device_wanted.device_eui) {
                update_json(device_wanted, &mut devices_json["devices"][index])?;
                included = true;
            }

            index += 1;
        }

        if !included {
            devices_json["devices"].push(create_json(device_wanted))?;
        }
    }

    curl::put(get_url(token, "loraNetwork/whitelist"), devices_json)?;

    save_apply(token)?;

    Ok(())
}

pub fn remove(token: &Token, devices: &[Eui]) -> Result<(), MtcapError> {
    let gateway_response = curl::get(get_url(token, "loraNetwork/whitelist"))?;
    let mut allowlist_json = json::parse(&gateway_response)?["result"].clone();
    let mut index = 0;
    while !allowlist_json["devices"][index].is_null() {
        let device_eui_existing =
            Eui::from_str(&allowlist_json["devices"][index]["deveui"].to_string())?;
        if devices.contains(&device_eui_existing) {
            allowlist_json["devices"].array_remove(index);
        } else {
            index += 1;
        }
    }

    curl::put(get_url(token, "loraNetwork/whitelist"), allowlist_json)?;

    let gateway_response = curl::get(get_url(token, "lora/devices"))?;
    let devices_json = json::parse(&gateway_response)?["result"].clone();
    let mut index = 0;
    while !devices_json[index].is_null() {
        let device_eui_existing = Eui::from_str(&devices_json[index]["deveui"].to_string())?;
        if devices.contains(&device_eui_existing) {
            curl::delete(get_url(
                token,
                format!("lora/devices/{device_eui_existing}"),
            ))?;
        }

        index += 1;
    }

    save_apply(token)?;

    Ok(())
}

pub fn remove_old(token: &Token, older_than: chrono::NaiveDate) -> Result<(), MtcapError> {
    let mut devices_to_remove = Vec::new();

    let gateway_response = curl::get(get_url(token, "lora/devices"))?;
    let devices_json = json::parse(&gateway_response)?["result"].clone();
    if let json::JsonValue::Array(devices_array) = devices_json {
        for device in devices_array {
            let device_date = if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(
                &device["last_seen"].to_string(),
                "%Y-%m-%dT%H:%M:%SZ",
            ) {
                dt.date()
            } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(
                &device["created_at"].to_string(),
                "%Y-%m-%dT%H:%M:%SZ",
            ) {
                dt.date()
            } else {
                continue;
            };
            if device_date < older_than {
                devices_to_remove.push(Eui::from_str(&device["deveui"].to_string())?);
            }
        }
    }

    remove(token, &devices_to_remove)
}

fn create_json(device: &Device) -> json::JsonValue {
    json::object! {
        deveui: device.device_eui.to_string(),
        appeui: device.join_eui.to_string(),
        appkey: device.application_key.to_string_no_spaces(),
        class: device.class.to_string(),
        device_profile_id: format!("LW102-OTA-{}", device.device_profile),
        network_profile_id: format!("DEFAULT-CLASS-{}", device.network_profile),
    }
}

fn update_json(device: &Device, json: &mut json::JsonValue) -> Result<(), MtcapError> {
    json["deveui"] = device.device_eui.to_string().into();
    json["appeui"] = device.join_eui.to_string().into();
    json["appkey"] = device.application_key.to_string_no_spaces().into();
    json["class"] = device.class.to_string().into();
    json["device_profile_id"] = format!("LW102-OTA-{}", device.device_profile).into();
    json["network_profile_id"] = format!("DEFAULT-CLASS-{}", device.network_profile).into();

    Ok(())
}

#[cfg(test)]
#[path = "./test_devices.rs"]
mod test_devices;

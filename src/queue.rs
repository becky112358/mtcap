use crate::credentials::{get_url, save_apply, Token};
use crate::curl;
use crate::devices::Eui;
use crate::result::MtcapError;

#[derive(Clone, Debug)]
pub struct Packet {
    data: String,
    device_eui: Eui,
    port: u8,
}

impl Packet {
    pub fn device_eui(&self) -> &Eui {
        &self.device_eui
    }

    pub fn port(&self) -> u8 {
        self.port
    }

    pub fn data(&self) -> &str {
        &self.data
    }
}

pub fn get(token: &Token) -> Result<Vec<Packet>, MtcapError> {
    let gateway_response = curl::get(get_url(token, "lora/packets/queue"))?;
    let packets_json = &json::parse(&gateway_response)?["result"];

    let mut packets = Vec::new();

    let mut index = 0;
    while !packets_json[index].is_null() {
        let packet = extract_json(&packets_json[index])?;
        packets.push(packet);
        index += 1;
    }

    Ok(packets)
}

pub fn remove(token: &Token, device_euis: &[Eui]) -> Result<(), MtcapError> {
    for device_eui in device_euis {
        curl::delete(get_url(token, format!("lora/packets/queue/{device_eui}")))?;
    }

    save_apply(token)?;

    Ok(())
}

fn extract_json(json: &json::JsonValue) -> Result<Packet, MtcapError> {
    Ok(Packet {
        data: json["data"].to_string(),
        device_eui: json["deveui"].to_string().parse()?,
        port: json["port"].to_string().parse()?,
    })
}

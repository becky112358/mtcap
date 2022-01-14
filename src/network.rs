use anyhow::Result;

use crate::credentials::{get_url, save_apply, Token};
use crate::curl;

pub enum Mode {
    NetworkServer,
    PacketForwarder,
    Disabled,
}

pub fn set_mode(token: &Token, mode: Mode) -> Result<()> {
    let response = curl::get(get_url(token, "loraNetwork/lora"))?;

    let mut json = json::parse(&response)?;
    json["enabled"] = match mode {
        Mode::NetworkServer => json::JsonValue::Boolean(true),
        Mode::PacketForwarder => json::JsonValue::Boolean(true),
        Mode::Disabled => json::JsonValue::Boolean(false),
    };
    json["packetForwarderMode"] = match mode {
        Mode::NetworkServer => json::JsonValue::Boolean(false),
        Mode::PacketForwarder => json::JsonValue::Boolean(true),
        Mode::Disabled => json::JsonValue::Boolean(false),
    };

    curl::put(get_url(token, "loraNetwork/lora"), json)?;

    save_apply(token)?;

    Ok(())
}

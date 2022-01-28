use std::process::{Command, Output};

use crate::result::MtcapError;

pub fn get(url: String) -> Result<String, MtcapError> {
    let response = Command::new("curl").arg("-k").arg(url).output()?;

    response_analyse(&response)?;

    Ok(String::from_utf8_lossy(&response.stdout).to_string())
}

pub fn post(url: String) -> Result<(), MtcapError> {
    let response = Command::new("curl")
        .arg("-k")
        .arg(url)
        .arg("-X")
        .arg("POST")
        .arg("-d")
        .arg("\"\"")
        .output()?;

    response_analyse(&response)?;

    Ok(())
}

pub fn put(url: String, json: json::JsonValue) -> Result<(), MtcapError> {
    let response = Command::new("curl")
        .arg("-k")
        .arg(url)
        .arg("-X")
        .arg("PUT")
        .arg("-d")
        .arg(json::stringify(json))
        .arg("-H")
        .arg("Content-Type: application/json")
        .output()?;

    response_analyse(&response)?;

    Ok(())
}

fn response_analyse(output: &Output) -> Result<(), MtcapError> {
    let json = json::parse(&String::from_utf8_lossy(&output.stdout))?;
    let status = json["status"].to_string();

    if status.eq("success") {
        Ok(())
    } else if !json["error"].is_null() {
        Err(MtcapError::Other(json["error"].to_string()))
    } else {
        Err(MtcapError::Other(format!("{:?}", output)))
    }
}

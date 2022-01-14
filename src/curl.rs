use std::process::{Command, Output};

use anyhow::Result;

pub fn get(url: String) -> Result<String> {
    let response = Command::new("curl").arg("-k").arg(url).output()?;

    response_analyse(&response)?;

    Ok(String::from_utf8_lossy(&response.stdout).to_string())
}

pub fn post(url: String) -> Result<()> {
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

pub fn put(url: String, json: json::JsonValue) -> Result<()> {
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

fn response_analyse(output: &Output) -> Result<()> {
    let json = json::parse(&String::from_utf8_lossy(&output.stdout))?;
    let status = json["status"].to_string();

    if status.eq("success") {
        Ok(())
    } else if !json["error"].is_null() {
        println!("Gateway error");
        Err(anyhow::anyhow!(json["error"].to_string()))
    } else {
        println!("Gateway error");
        Err(anyhow::anyhow!(format!("{:?}", output)))
    }
}

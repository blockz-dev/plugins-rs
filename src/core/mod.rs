mod fix;
mod dirs;

use crate::types::Output;
use deno_core::{ extension, op2 };

use std::usize;

#[op2]
#[serde]
fn js_which(
    #[string] path: String
) -> std::io::Result<Output> {
    let w = fix::rust_which(path).unwrap();
    Ok(Output {
        data: format!("{}", w.display()).into(),
    })
}


#[op2]
#[serde]
fn nid(
    #[smi] size: usize
) -> std::io::Result<Output> {
    Ok(Output {
        data: nanoid::nanoid!(size).into(),
    })
}


#[op2]
#[serde]
fn nid_custom(
    #[smi] size: usize,
    #[string] custom: String
) -> std::io::Result<Output> {
    let mut v = vec![];
    custom.chars().for_each(|c| v.push(c));
    Ok(Output {
        data: nanoid::nanoid!(size, &v).into(),
    })
}


#[op2]
#[serde]
fn nid_safe(
    #[smi] size: usize,
) -> std::io::Result<Output> {
    Ok(Output {
        data: nanoid::nanoid!(size, &nanoid::alphabet::SAFE, random).into(),
    })
}


#[op2]
#[serde]
fn uid() -> std::io::Result<Output> {
    let uuid = uuid::Uuid::new_v4();
    Ok(Output { data: uuid.to_string() })
}





static CONFIG_TOML : &str = include_str!("../../Cargo.toml");

#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigToml {
    package: ConfigTomlPackage,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigTomlPackage {
    name: String,
    version: String,
    edition: String,
}



#[op2]
#[serde]
fn config() -> std::io::Result<ConfigToml> {
    let config: ConfigToml = toml::from_str(CONFIG_TOML).unwrap();
    Ok(config)
}








use log::Level::{Error, Warn, Info, Debug, Trace};
use ansi_term::{ANSIGenericString, Colour::{Red, Blue, Yellow, White, Cyan}};

#[cfg(target_os = "windows")]
pub fn enable_ansi() {
    match ansi_term::enable_ansi_support() {
        Ok(_) => {},
        Err(e) => println!("{}", e),
    }
}

//#[allow(clippy::match_single_binding)] // needed for temporary lifetime
#[op2(fast)]
fn op_internal_log(
    #[string] url: String,
    #[smi] level: u32,
    #[string] message: String,
) {
    let level = match level {
        1 => log::Level::Error.as_str(),
        2 => log::Level::Warn.as_str(),
        3 => log::Level::Info.as_str(),
        4 => log::Level::Debug.as_str(),
        5 => log::Level::Trace.as_str(),
        _ => unreachable!(),
    };
    println!("[{}] {} {}", level, &url, message);
}

//#[allow(clippy::match_single_binding)] // needed for temporary lifetime
#[op2(fast)]
fn op_internal_color_log(
    #[string] url: String,
    #[smi] level: u32,
    #[string] message: String,
) {
    let res = match level {
        1 => Red.bold().paint(format!("[{}] {} {}", Error.as_str(), &url, message)),
        2 => Yellow.bold().paint(format!("[{}] {} {}", Warn.as_str(), &url, message)),
        3 => Cyan.bold().paint(format!("[{}] {} {}", Info.as_str(), &url, message)),
        4 => Blue.bold().paint(format!("[{}] {} {}", Debug.as_str(), &url, message)),
        5 => White.bold().paint(format!("[{}] {} {}", Trace.as_str(), &url, message)),
        _ => unreachable!(),
    };
    println!("{}", res);
}




fn _result_format<'a>(level: u32, url: &'a str, message: &'a str) -> ANSIGenericString<'a, str> {
    match level {
        1 => Red.paint(format!("[{}] {} {}", Error.as_str(), &url, message)),
        2 => Yellow.paint(format!("[{}] {} {}", Warn.as_str(), &url, message)),
        3 => Cyan.paint(format!("[{}] {} {}", Info.as_str(), &url, message)),
        4 => Blue.paint(format!("[{}] {} {}", Debug.as_str(), &url, message)),
        5 => White.paint(format!("[{}] {} {}", Trace.as_str(), &url, message)),
        _ => unreachable!(),
    }
}





extension!(
    core_js,
    ops = [
        uid,
        nid,
        nid_custom,
        nid_safe,
        js_which,
        config,

        op_internal_log,
        op_internal_color_log,

        dirs::audio_dir,
        dirs::cache_dir,
        dirs::config_dir,
        dirs::config_local_dir,
        dirs::data_dir,
        dirs::data_local_dir,
        dirs::desktop_dir,
        dirs::document_dir,
        dirs::download_dir,
        dirs::home_dir,
        dirs::picture_dir,
        dirs::video_dir,
    ],
    esm_entry_point = "plugins:core", 
    esm = [
        dir "src/core",
        "plugins:core" = "internal.js"
    ],
    docs = "Rust Based HTML Scraper", "scraper html from rust"
);

pub fn init() -> deno_core::Extension {
    core_js::init()
}




















//////////////////////
/// 
/// 

fn random (size: usize) -> Vec<u8> {
    let result: Vec<u8> = vec![0; size];
    result
}
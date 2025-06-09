use std::fs;
use knus::{Decode, DecodeScalar, Error};
use crate::config::ReadConfigError::{IoError, ParseError};

#[derive(Decode, Clone, Debug)]
pub struct Admin {
    #[knus(child, unwrap(argument))]
    pub username: String,
    #[knus(child, unwrap(argument))]
    pub password: String,
}
#[derive(Decode, Clone, Debug)]
pub struct Application {
    #[knus(child, unwrap(argument))]
    pub id: String,
    #[knus(child, unwrap(argument))]
    pub name: String,
    #[knus(child)]
    pub oauth: OAuth2
}

#[derive(Decode, Clone, Debug)]
pub struct OAuth2 {
    #[knus(child, unwrap(argument))]
    pub client_secret: String,
    #[knus(child, unwrap(arguments))]
    pub authorized_origin_urls: Vec<String>,
    #[knus(child, unwrap(arguments))]
    pub authorized_redirect_urls: Vec<String>,
    #[knus(child, unwrap(argument))]
    pub logout_url: String,
}

#[derive(Decode, Debug)]
pub struct Config {
    #[knus(child, unwrap(argument))]
    pub base_path: String,
    #[knus(child, unwrap(argument))]
    pub api_key: String,
    #[knus(child, unwrap(argument))]
    pub external_url: String,
    #[knus(child, unwrap(argument))]
    pub issuer: String,
    #[knus(child)]
    pub admin: Admin,
    #[knus(child)]
    pub application: Application,
}

#[derive(thiserror::Error, Debug)]
pub enum ReadConfigError {
    #[error(transparent)]
    IoError(std::io::Error),
    #[error(transparent)]
    ParseError(Error),
}

pub fn read_config(path: &str) -> Result<Config, ReadConfigError> {
    let config_str = fs::read_to_string(path).map_err(IoError)?;
    knus::parse::<Config>(path, &config_str).map_err(ParseError)
}
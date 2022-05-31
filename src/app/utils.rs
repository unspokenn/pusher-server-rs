use hex::{FromHex, ToHex};
use sha2::Sha256;
use hmac::{Hmac, Mac};
use crate::app::CustomError;

type HmacSha256 = Hmac<Sha256>;

#[allow(dead_code)]
#[inline(always)]
pub(crate) fn create_body_md5(body: &str) -> String {
    use md5::{Md5, Digest};

    let mut sh = Md5::new();
    sh.update(body.as_bytes());
    sh.finalize().encode_hex()
}

#[allow(dead_code)]
#[inline(always)]
pub(crate) fn create_channel_auth<'a>(auth_map: &mut hashbrown::HashMap<&'a str, String>, key: &str, secret: &str, to_sign: &str) {
    let auth_signature = create_auth_signature(to_sign, secret);
    let auth_string = format!("{}:{}", key, auth_signature);
    auth_map.insert("auth", auth_string);
}

#[inline(always)]
pub(crate) fn check_signature(signature: &str, secret: &str, body: &str) -> Result<(), warp::Rejection> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(body.as_bytes());

    if let Ok(decoded_signature) = Vec::from_hex(signature) {
        if let Ok(_) = mac.verify_slice(&decoded_signature[..]) {
            Ok(())
        } else { Err(warp::reject::custom(CustomError::AuthKeyMismatch)) }
    } else { Err(warp::reject::custom(CustomError::AuthSignatureError)) }
}

#[allow(dead_code)]
#[inline(always)]
pub(crate) fn create_auth_signature(to_sign: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(to_sign.as_bytes());
    mac.finalize().into_bytes().encode_hex()
}

#[inline(always)]
pub(crate) fn generate_socket_id() -> String {
    use rand::distributions::{Distribution, Uniform};
    let digits = Uniform::from(0..=9)
        .sample_iter(&mut rand::thread_rng())
        .take(32)
        .fold(String::from(""), |res, digit| format!("{}{}", res, digit));

    let (p1, p2) = digits.split_at(16);
    format!("{}.{}", p1, p2)
}

#[allow(dead_code)]
#[inline(always)]
pub(crate) fn validate_channels(channels: &Vec<String>) -> Result<bool, String> {
    if channels.len() > 10 {
        return Err("Cannot trigger on more than 10 channels".to_string());
    }

    let channel_regex = regex::Regex::new(r"^[-a-zA-Z0-9_=@,.;]+$").unwrap();

    for channel in channels {
        if channel.len() > 200 {
            return Err("Channel names must be under 200 characters".to_string());
        }
        if !channel_regex.is_match(channel) {
            return Err("Channels must be formatted as such: ^[-a-zA-Z0-9_=@,.;]+$".to_string());
        }
    }
    Ok(true)
}

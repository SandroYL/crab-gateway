use std::ops::Deref;

use http::Version;


#[inline]
pub fn get_version_str(version: &Version) -> String {
    match *version {
        Version::HTTP_09 => "HTTP/0.9".to_string(),
        Version::HTTP_10 => "HTTP/1.0".to_string(),
        Version::HTTP_11 => "HTTP/1.1".to_string(),
        Version::HTTP_2 => "HTTP/2.0".to_string(),
        Version::HTTP_3 => "HTTP/3.0".to_string(),
        _ => "HTTP/1.0".to_string()
    }
}
use std::fs::File;
use std::io::BufReader;

use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};

pub fn configure<C: AsRef<str>, P: AsRef<str>>(cert: C, key: P) -> ServerConfig {
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    let cert_file = &mut BufReader::new(File::open(cert.as_ref()).unwrap());
    let key_file = &mut BufReader::new(File::open(key.as_ref()).unwrap());

    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect::<Vec<_>>();
    let mut keys = pkcs8_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect::<Vec<_>>();

    if keys.is_empty() {
        panic!("No private key found");
    }

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}

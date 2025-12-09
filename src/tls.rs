use std::fs::File;
use std::io::BufReader;

use rustls::ServerConfig;
use rustls::pki_types::PrivateKeyDer;
use rustls_pemfile::{certs, pkcs8_private_keys};

pub fn configure<C: AsRef<str>, P: AsRef<str>>(cert: C, key: P) -> ServerConfig {
    let config = ServerConfig::builder().with_no_client_auth();

    let cert_file = &mut BufReader::new(File::open(cert.as_ref()).unwrap());
    let key_file = &mut BufReader::new(File::open(key.as_ref()).unwrap());

    let cert_chain = certs(cert_file).collect::<Result<Vec<_>, _>>().unwrap();
    let mut keys = pkcs8_private_keys(key_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    if keys.is_empty() {
        panic!("No private key found");
    }

    config
        .with_single_cert(cert_chain, PrivateKeyDer::Pkcs8(keys.remove(0)))
        .unwrap()
}

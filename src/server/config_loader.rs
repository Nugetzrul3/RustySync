use std::fs::File;
use std::io::BufReader;
use rustls::crypto::aws_lc_rs;
use rustls::pki_types::PrivateKeyDer;
use rustls::ServerConfig;

pub fn load_config() -> Option<ServerConfig> {
    aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut certs_file = BufReader::new(File::open("certs/cert.pem").ok()?);
    let mut keys_file = BufReader::new(File::open("certs/key.pem").ok()?);

    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>().ok()?;
    let mut tls_key = rustls_pemfile::pkcs8_private_keys(&mut keys_file)
        .collect::<Result<Vec<_>, _>>().ok()?;

    if tls_key.is_empty() {
        eprintln!("No TLS keys found");
        return None;
    }

    let tls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, PrivateKeyDer::Pkcs8(tls_key.remove(0)))
        .ok()?;

    Some(tls_config)
}
//! Loopback-only authentication fixture; not the Android daemon.
use altdb::{
    crypto::{Identity, Tls, host_key},
    pairing,
    protocol::*,
};
use anyhow::Result;
use std::{io::Write, net::TcpListener};

fn main() -> Result<()> {
    let identity = Identity::generate()?;
    let pair = TcpListener::bind("127.0.0.1:39001")?;
    let connect = TcpListener::bind("127.0.0.1:39002")?;
    println!("LOOPBACK TEST: adb pair 127.0.0.1:39001 123456");
    let key = pairing::pair(pair.accept()?.0, &identity, "123456")?;
    let (fp, _) = host_key(&key)?;
    println!("PAIRED: {fp}");
    let (mut socket, _) = connect.accept()?;
    let cnxn = Packet::read(&mut socket, 4096, false)?;
    anyhow::ensure!(cnxn.command == CNXN, "expected CNXN");
    Packet::new(STLS, STLS_VERSION, 0, vec![]).write(&mut socket, false)?;
    anyhow::ensure!(
        Packet::read(&mut socket, 4096, false)?.command == STLS,
        "expected STLS"
    );
    let mut tls = Tls::accept(socket, &identity, &[fp], false)?;
    Packet::new(
        CNXN,
        VERSION,
        MAX_PAYLOAD as u32,
        b"device::ro.product.name=altdb_auth_fixture;features=;\0".to_vec(),
    )
    .write(&mut tls, false)?;
    tls.flush()?;
    println!("TLS AUTHENTICATED");
    loop {
        let packet = Packet::read(&mut tls, MAX_PAYLOAD, false)?;
        if packet.command == OPEN {
            Packet::new(CLSE, 0, packet.arg0, vec![]).write(&mut tls, false)?;
            tls.flush()?;
        }
    }
}

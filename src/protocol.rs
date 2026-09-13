use std::io::{self, Read, Write};

pub const VERSION: u32 = 0x01000001;
pub const STLS_VERSION: u32 = 0x01000000;
pub const MAX_PAYLOAD: usize = 1024 * 1024;
pub const CHUNK: usize = 64 * 1024;
pub const CNXN: u32 = id(b"CNXN");
pub const STLS: u32 = id(b"STLS");
pub const OPEN: u32 = id(b"OPEN");
pub const OKAY: u32 = id(b"OKAY");
pub const WRTE: u32 = id(b"WRTE");
pub const CLSE: u32 = id(b"CLSE");
pub const FEATURES: &str = "shell_v2,cmd,stat_v2,ls_v2,sendrecv_v2";
pub const fn id(s: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*s)
}

pub fn invalid(s: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub command: u32,
    pub arg0: u32,
    pub arg1: u32,
    pub data: Vec<u8>,
}

impl Packet {
    pub fn new(command: u32, arg0: u32, arg1: u32, data: impl Into<Vec<u8>>) -> Self {
        Self {
            command,
            arg0,
            arg1,
            data: data.into(),
        }
    }

    pub fn read(r: &mut impl Read, max: usize, checksum: bool) -> io::Result<Self> {
        let mut h = [0u8; 24];
        r.read_exact(&mut h)?;
        let fields: Vec<_> = h
            .chunks_exact(4)
            .map(|s| u32::from_le_bytes(s.try_into().unwrap()))
            .collect();
        if fields[0] ^ fields[5] != u32::MAX {
            return Err(invalid("bad ADB magic"));
        }
        let len = fields[3] as usize;
        if len > max.min(MAX_PAYLOAD) {
            return Err(invalid("ADB packet exceeds limit"));
        }
        let mut data = vec![0; len];
        r.read_exact(&mut data)?;
        if checksum && data.iter().fold(0u32, |a, x| a.wrapping_add(*x as u32)) != fields[4] {
            return Err(invalid("bad ADB checksum"));
        }
        Ok(Self::new(fields[0], fields[1], fields[2], data))
    }

    pub fn write(&self, w: &mut impl Write, checksum: bool) -> io::Result<()> {
        if self.data.len() > MAX_PAYLOAD {
            return Err(invalid("ADB packet exceeds limit"));
        }
        let sum = if checksum {
            self.data
                .iter()
                .fold(0u32, |a, x| a.wrapping_add(*x as u32))
        } else {
            0
        };
        // One ADB packet per write: splitting even the six header fields into
        // separate TLS writes produces tiny records and delayed-ACK stalls.
        let mut packet = Vec::with_capacity(24 + self.data.len());
        for n in [
            self.command,
            self.arg0,
            self.arg1,
            self.data.len() as u32,
            sum,
            self.command ^ u32::MAX,
        ] {
            packet.extend_from_slice(&n.to_le_bytes());
        }
        packet.extend_from_slice(&self.data);
        w.write_all(&packet)
    }
}

pub fn shell_frame(w: &mut impl Write, channel: u8, data: &[u8]) -> io::Result<()> {
    w.write_all(&[channel])?;
    w.write_all(&(data.len() as u32).to_le_bytes())?;
    w.write_all(data)
}

pub fn shell_read(r: &mut impl Read) -> io::Result<(u8, Vec<u8>)> {
    let mut h = [0; 5];
    r.read_exact(&mut h)?;
    let len = u32::from_le_bytes(h[1..].try_into().unwrap()) as usize;
    if len > MAX_PAYLOAD {
        return Err(invalid("shell frame exceeds limit"));
    }
    let mut data = vec![0; len];
    r.read_exact(&mut data)?;
    Ok((h[0], data))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packet_roundtrip_and_truncation() {
        let p = Packet::new(WRTE, 1, 2, b"hello\0\xff".to_vec());
        let mut b = vec![];
        p.write(&mut b, true).unwrap();
        assert_eq!(p, Packet::read(&mut &b[..], MAX_PAYLOAD, true).unwrap());
        for end in 0..b.len() {
            assert!(Packet::read(&mut &b[..end], MAX_PAYLOAD, true).is_err());
        }
        b[20] ^= 1;
        assert!(Packet::read(&mut &b[..], MAX_PAYLOAD, true).is_err());
    }
    #[test]
    fn oversized_header_rejected_before_allocation() {
        let mut b = vec![];
        Packet::new(WRTE, 1, 2, vec![])
            .write(&mut b, false)
            .unwrap();
        b[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Packet::read(&mut &b[..], MAX_PAYLOAD, false).is_err());
    }
    #[test]
    fn shell_binary_frame() {
        let mut b = vec![];
        shell_frame(&mut b, 2, b"\0\xff").unwrap();
        assert_eq!(shell_read(&mut &b[..]).unwrap(), (2, vec![0, 255]));
    }
}

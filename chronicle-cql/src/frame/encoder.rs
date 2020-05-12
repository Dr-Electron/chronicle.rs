use std::net::{IpAddr,Ipv4Addr, Ipv6Addr};

pub const BE_16_BYTES_LEN: [u8; 4] = [0,0,0,16];
pub const BE_8_BYTES_LEN: [u8; 4] = [0,0,0,8];
pub const BE_4_BYTES_LEN: [u8; 4] = [0,0,0,4];
pub const BE_2_BYTES_LEN: [u8; 4] = [0,0,0,2];
pub const BE_1_BYTES_LEN: [u8; 4] = [0,0,0,1];
pub const BE_0_BYTES_LEN: [u8; 4] = [0,0,0,0];
pub const BE_NULL_BYTES_LEN: [u8; 4] = [255,255,255,255]; // -1 length
pub const BE_UNSET_BYTES_LEN: [u8; 4] = [255,255,255,254]; // -2 length
pub const NULL_VALUE: Null = Null;
pub const UNSET_VALUE: Unset = Unset;
pub struct Null;
pub struct Unset;

pub trait ColumnEncoder {
    fn encode(&self, buffer: &mut Vec<u8>);
}

impl ColumnEncoder for i64 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_8_BYTES_LEN);
        buffer.extend(&i64::to_be_bytes(*self));
    }
}

impl ColumnEncoder for u64 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_8_BYTES_LEN);
        buffer.extend(&u64::to_be_bytes(*self));
    }
}

impl ColumnEncoder for f64 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_8_BYTES_LEN);
        buffer.extend(&f64::to_be_bytes(*self));
    }
}

impl ColumnEncoder for i32 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_4_BYTES_LEN);
        buffer.extend(&i32::to_be_bytes(*self));
    }
}

impl ColumnEncoder for u32 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_4_BYTES_LEN);
        buffer.extend(&u32::to_be_bytes(*self));
    }
}

impl ColumnEncoder for f32 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_4_BYTES_LEN);
        buffer.extend(&f32::to_be_bytes(*self));
    }
}

impl ColumnEncoder for i16 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_2_BYTES_LEN);
        buffer.extend(&i16::to_be_bytes(*self));
    }
}

impl ColumnEncoder for u16 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_2_BYTES_LEN);
        buffer.extend(&u16::to_be_bytes(*self));
    }
}

impl ColumnEncoder for i8 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_1_BYTES_LEN);
        buffer.extend(&i8::to_be_bytes(*self));
    }
}

impl ColumnEncoder for u8 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_1_BYTES_LEN);
        buffer.push(*self);
    }
}

impl ColumnEncoder for bool {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_1_BYTES_LEN);
        buffer.push(*self as u8);
    }
}

impl ColumnEncoder for String {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&i32::to_be_bytes(self.len() as i32));
        buffer.extend(self.bytes());
    }
}
impl ColumnEncoder for &str {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&i32::to_be_bytes(self.len() as i32));
        buffer.extend(self.bytes());
    }
}
impl ColumnEncoder for &[u8] {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&i32::to_be_bytes(self.len() as i32));
        buffer.extend(*self);
    }
}

impl ColumnEncoder for IpAddr {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            &IpAddr::V4(ip) => {
                buffer.extend(&BE_4_BYTES_LEN);
                buffer.extend(&ip.octets());
            }
            &IpAddr::V6(ip) => {
                buffer.extend(&BE_16_BYTES_LEN);
                buffer.extend(&ip.octets());
            }
        }
    }
}

impl ColumnEncoder for Ipv4Addr {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_4_BYTES_LEN);
        buffer.extend(&self.octets());
    }
}

impl ColumnEncoder for Ipv6Addr {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_16_BYTES_LEN);
        buffer.extend(&self.octets());
    }
}

impl ColumnEncoder for Unset {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&BE_UNSET_BYTES_LEN);
    }
}
extern crate bytes;

use bytes::{BufMut, BytesMut};
use std::time::{SystemTime, UNIX_EPOCH};

// TODO: unmarshal
pub enum ColferTypes<'a> {
    B(bool),
    U32(u32),
    U64(u64),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    T(SystemTime),
    S(&'a str),
    A(&'a [u8]),
    O(Box<ColferTypes<'a>>),
    Os(Vec<ColferTypes<'a>>),
    Ss(&'a [&'a str]),
    As(&'a [&'a [u8]]),
    U8(u8),
    U16(u16),
    F32s(&'a [f32]),
    F64s(&'a [f64]),
}

impl<'a> ColferTypes<'a> {
    pub fn marshal_to(&self, buf: &mut BytesMut) {
        match self {
            ColferTypes::B(_) => {
                buf.put_u8(0);
            }
            ColferTypes::U32(mut x) if x != 0 => {
                if x >= 1 << 21 {
                    buf.put_u8(1 | 0x80);
                    buf.put_u32_be(x);
                } else {
                    buf.put_u8(1);
                    while x >= 0x80 {
                        buf.put_u8(x as u8 | 0x80);
                        x >>= 7;
                    }
                    buf.put_u8(x as u8);
                }
            }
            ColferTypes::U64(mut x) if x != 0 => {
                if x >= 1 << 49 {
                    buf.put_u8(2 | 0x80);
                    buf.put_u64_be(x);
                } else {
                    buf.put_u8(2);
                    while x >= 0x80 {
                        buf.put_u8(x as u8 | 0x80);
                        x >>= 7;
                    }
                    buf.put_u8(x as u8);
                }
            }
            ColferTypes::I32(v) if *v != 0 => {
                let mut x = *v as u32;
                if *v > 0 {
                    buf.put_u8(3);
                } else {
                    x = x ^ x + 1;
                    buf.put_u8(3 | 0x80);
                }

                while x >= 0x80 {
                    buf.put_u8(x as u8 | 0x80);
                    x >>= 7;
                }

                buf.put_u8(x as u8);
            }
            ColferTypes::I64(v) if *v != 0 => {
                let mut x = *v as u64;
                if *v > 0 {
                    buf.put_u8(4);
                } else {
                    x = x ^ x + 1;
                    buf.put_u8(4 | 0x80);
                }

                for _ in 0..8 {
                    if x < 0x80 {
                        break;
                    }

                    buf.put_u8(x as u8 | 0x80);
                    x >>= 7;
                }

                buf.put_u8(x as u8);
            }
            ColferTypes::F32(v) if *v != 0.0 => {
                buf.put_u8(5);
                buf.put_u32_be(v.to_bits());
            }
            ColferTypes::F64(v) if *v != 0.0 => {
                buf.put_u8(5);
                buf.put_u64_be(v.to_bits());
            }
            ColferTypes::T(v) if *v != UNIX_EPOCH => {
                // Safe unwrap since we checked if it didn't match UNIX_EPOCH
                let dur = v.duration_since(UNIX_EPOCH).unwrap();
                let s = dur.as_secs();
                if s < 1 << 32 {
                    buf.put_u8(7);
                    buf.put_u32_be(s as u32);
                } else {
                    buf.put_u8(7 | 0x80);
                    buf.put_u64_be(s);
                }

                buf.put_u32_be(dur.subsec_nanos());
            }
            ColferTypes::S(v) if v.len() > 0 => {
                buf.put_u8(8);
                let mut x = v.len();
                while x >= 0x80 {
                    buf.put_u8(x as u8 | 0x80);
                    x >>= 7;
                }
                buf.put_u8(x as u8);
                buf.put_slice(&v.as_bytes());
            }
            ColferTypes::A(v) if v.len() > 0 => {
                buf.put_u8(9);
                let mut x = v.len();
                while x >= 0x80 {
                    buf.put_u8(x as u8 | 0x80);
                    x >>= 7;
                }
                buf.put_u8(x as u8);
                buf.put_slice(&v);
            }
            ColferTypes::O(v) => {
                buf.put_u8(10);
                v.marshal_to(buf);
            }
            ColferTypes::Os(v) => {
                let len = v.len();
                if len > 0 {
                    buf.put_u8(11);
                    let mut x = len as u32;
                    while x >= 0x80 {
                        buf.put_u8(x as u8 | 0x80);
                        x >>= 7;
                    }
                    buf.put_u8(x as u8);

                    for vi in v.iter() {
                        vi.marshal_to(buf);
                    }
                }
            }
            ColferTypes::Ss(v) if v.len() > 0 => {
                buf.put_u8(12);
                let mut x = v.len() as u32;
                while x >= 0x80 {
                    buf.put_u8(x as u8 | 0x80);
                    x >>= 7;
                }
                buf.put_u8(x as u8);

                for s in v.iter() {
                    let mut xs = s.len() as u32;
                    while xs >= 0x80 {
                        buf.put_u8(xs as u8 | 0x80);
                        xs >>= 7;
                    }
                    buf.put_u8(xs as u8);

                    buf.put_slice(&s.as_bytes());
                }
            }
            ColferTypes::As(v) if v.len() > 0 => {
                buf.put_u8(13);
                let mut x = v.len() as u32;
                while x >= 0x80 {
                    buf.put_u8(x as u8 | 0x80);
                    x >>= 7;
                }
                buf.put_u8(x as u8);

                for a in v.iter() {
                    let mut xs = a.len() as u32;
                    while xs >= 0x80 {
                        buf.put_u8(xs as u8 | 0x80);
                        xs >>= 7;
                    }
                    buf.put_u8(xs as u8);

                    buf.put_slice(&a);
                }
            }
            ColferTypes::U8(x) if *x > 0 => {
                buf.put_u8(14);
                buf.put_u8(*x);
            }
            ColferTypes::U16(x) if *x > 0 => {
                if *x >= 1 << 8 {
                    buf.put_u8(15);
                    buf.put_u8(((*x) >> 8) as u8);
                    buf.put_u8(*x as u8);
                } else {
                    buf.put_u8(15 | 0x80);
                    buf.put_u8(*x as u8);
                }
            }
            ColferTypes::F32s(v) if v.len() > 0 => {
                buf.put_u8(16);
                let mut x = v.len();
                while x >= 0x80 {
                    buf.put_u8(x as u8 | 0x80);
                    x >>= 7;
                }
                buf.put_u8(x as u8);
                for f in v.iter() {
                    buf.put_u32_be(f.to_bits());
                }
            }
            ColferTypes::F64s(v) if v.len() > 0 => {
                buf.put_u8(16);
                let mut x = v.len();
                while x >= 0x80 {
                    buf.put_u8(x as u8 | 0x80);
                    x >>= 7;
                }
                buf.put_u8(x as u8);
                for f in v.iter() {
                    buf.put_u64_be(f.to_bits());
                }
            }
            _ => {}
        }

        buf.put_u8(0x7F);
    }
}

// TODO:
pub struct Colfer {
    size_max: u32,
    list_max: u32,
}

impl Colfer {
    pub fn new() -> Self {
        Colfer {
            size_max: 16 * 1024 * 1024,
            list_max: 64 * 1024,
        }
    }

    pub fn size_max(&mut self, max: u32) {
        self.size_max = max;
    }

    pub fn list_max(&mut self, max: u32) {
        self.list_max = max;
    }
}

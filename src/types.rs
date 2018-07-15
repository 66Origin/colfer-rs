use bytes::{Buf, BufMut};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// TODO: unmarshal
use super::{error::{ColferError, ColferResult},
            ColferSerializable,
            COLFER_LIST_MAX,
            COLFER_SIZE_MAX};

macro_rules! buf_guard {
    ($buf:ident) => {
        if !$buf.has_remaining() {
            return Err(ColferError::UnexpectedEof);
        }
    };
}

/// Contains all supported data types.
#[allow(non_snake_case)]
pub struct ColferTypes<'a> {
    /// B tests booleans.
    B: bool,
    /// U32 tests unsigned 32-bit integers.
    U32: u32,
    /// U64 tests unsigned 64-bit integers.
    U64: u64,
    /// I32 tests signed 32-bit integers.
    I32: i32,
    /// I64 tests signed 64-bit integers.
    I64: i64,
    /// F64 tests 64-bit floating points.
    F64: f64,
    /// F32 tests 32-bit floating points.
    F32: f32,
    /// T tests timestamps.
    T: SystemTime,
    /// S tests text.
    S: &'a str,
    /// A tests binaries.
    A: Vec<u8>,
    /// O tests nested data structures.
    O: Option<Box<ColferTypes<'a>>>,
    /// Os tests data structure lists.
    Os: Vec<Option<ColferTypes<'a>>>,
    /// Ss tests text lists.
    Ss: Vec<&'a str>,
    /// As tests binary lists.
    As: Vec<&'a [u8]>,
    /// U8 tests unsigned 8-bit integers.
    U8: u8,
    /// U16 tests unsigned 16-bit integers.
    U16: u16,
    /// F32s tests 32-bit floating point lists.
    F32s: Vec<f32>,
    /// F64s tests 64-bit floating point lists.
    F64s: Vec<f64>,
}

impl<'a> Default for ColferTypes<'a> {
    fn default() -> Self {
        ColferTypes {
            B: bool::default(),
            U32: u32::default(),
            U64: u64::default(),
            I32: i32::default(),
            I64: i64::default(),
            F64: f64::default(),
            F32: f32::default(),
            T: SystemTime::now(),
            S: "",
            A: Vec::default(),
            O: Option::default(),
            Os: Vec::default(),
            Ss: Vec::default(),
            As: Vec::default(),
            U8: u8::default(),
            U16: u16::default(),
            F32s: Vec::default(),
            F64s: Vec::default(),
        }
    }
}

impl<'a> ColferSerializable<'a> for ColferTypes<'a> {
    // MarshalTo encodes o as Colfer into buf and returns the number of bytes written.
    fn colf_marshal_to(&self, buf: &mut Vec<u8>) -> usize {
        let previous_len = buf.len();
        if self.B {
            buf.put_u8(0);
        }

        {
            let mut x = self.U32;
            if x >= 1 << 21 {
                buf.put_u8(1 | 0x80);
                buf.put_u32_be(x);
            } else if x != 0 {
                buf.put_u8(1);
                while x >= 0x80 {
                    buf.put_u8(x as u8 | 0x80);
                    x >>= 7;
                }
                buf.put_u8(x as u8);
            }
        }

        {
            let mut x = self.U64;
            if x >= 1 << 49 {
                buf.put_u8(2 | 0x80);
                buf.put_u64_be(x);
            } else if x != 0 {
                buf.put_u8(2);
                while x >= 0x80 {
                    buf.put_u8(x as u8 | 0x80);
                    x >>= 7;
                }
                buf.put_u8(x as u8);
            }
        }

        if self.I32 != 0 {
            let mut x = self.I32 as u32;
            if self.I32 > 0 {
                buf.put_u8(3);
            } else {
                x = !x + 1;
                buf.put_u8(3 | 0x80);
            }

            while x >= 0x80 {
                buf.put_u8(x as u8 | 0x80);
                x >>= 7;
            }

            buf.put_u8(x as u8);
        }

        if self.I64 != 0 {
            let mut x = self.I64 as u64;
            if self.I64 > 0 {
                buf.put_u8(4);
            } else {
                x = !x + 1;
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

        if self.F32 != 0.0 {
            buf.put_u8(5);
            buf.put_u32_be(self.F32.to_bits());
        }

        if self.F64 != 0.0 {
            buf.put_u8(6);
            buf.put_u64_be(self.F64.to_bits());
        }

        if self.T != UNIX_EPOCH {
            // Safe unwrap since we checked if it didn't match UNIX_EPOCH
            let dur = self.T.duration_since(UNIX_EPOCH).unwrap();
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

        if self.S.len() > 0 {
            buf.put_u8(8);
            let mut x = self.S.len();
            while x >= 0x80 {
                buf.put_u8(x as u8 | 0x80);
                x >>= 7;
            }
            buf.put_u8(x as u8);
            buf.put_slice(&self.S.as_bytes());
        }

        if self.A.len() > 0 {
            buf.put_u8(9);
            let mut x = self.A.len();
            while x >= 0x80 {
                buf.put_u8(x as u8 | 0x80);
                x >>= 7;
            }
            buf.put_u8(x as u8);
            buf.put_slice(&self.A);
        }

        if let Some(ref o) = self.O {
            buf.put_u8(10);
            o.colf_marshal_to(buf);
        }

        if self.Os.len() > 0 {
            let len = self.Os.len();
            if len > 0 {
                buf.put_u8(11);
                let mut x = len as u32;
                while x >= 0x80 {
                    buf.put_u8(x as u8 | 0x80);
                    x >>= 7;
                }
                buf.put_u8(x as u8);

                for vio in self.Os.iter() {
                    if let Some(ref vi) = vio {
                        vi.colf_marshal_to(buf);
                    }
                }
            }
        }

        if self.Ss.len() > 0 {
            buf.put_u8(12);
            let mut x = self.Ss.len() as u32;
            while x >= 0x80 {
                buf.put_u8(x as u8 | 0x80);
                x >>= 7;
            }
            buf.put_u8(x as u8);

            for s in self.Ss.iter() {
                let mut xs = s.len() as u32;
                while xs >= 0x80 {
                    buf.put_u8(xs as u8 | 0x80);
                    xs >>= 7;
                }
                buf.put_u8(xs as u8);

                buf.put_slice(&s.as_bytes());
            }
        }
        if self.As.len() > 0 {
            buf.put_u8(13);
            let mut x = self.As.len() as u32;
            while x >= 0x80 {
                buf.put_u8(x as u8 | 0x80);
                x >>= 7;
            }
            buf.put_u8(x as u8);

            for a in self.As.iter() {
                let mut xs = a.len() as u32;
                while xs >= 0x80 {
                    buf.put_u8(xs as u8 | 0x80);
                    xs >>= 7;
                }
                buf.put_u8(xs as u8);

                buf.put_slice(&a);
            }
        }

        if self.U8 > 0 {
            buf.put_u8(14);
            buf.put_u8(self.U8);
        }

        if self.U16 >= 1 << 8 {
            buf.put_u8(15);
            buf.put_u8((self.U16 >> 8) as u8);
            buf.put_u8(self.U16 as u8);
        } else if self.U16 != 0 {
            buf.put_u8(15 | 0x80);
            buf.put_u8(self.U16 as u8);
        }

        if self.F32s.len() > 0 {
            buf.put_u8(16);
            let mut x = self.F32s.len();
            while x >= 0x80 {
                buf.put_u8(x as u8 | 0x80);
                x >>= 7;
            }
            buf.put_u8(x as u8);
            for f in self.F32s.iter() {
                buf.put_u32_be(f.to_bits());
            }
        }

        if self.F64s.len() > 0 {
            buf.put_u8(17);
            let mut x = self.F64s.len();
            while x >= 0x80 {
                buf.put_u8(x as u8 | 0x80);
                x >>= 7;
            }
            buf.put_u8(x as u8);
            for f in self.F64s.iter() {
                buf.put_u64_be(f.to_bits());
            }
        }

        buf.put_u8(0x7F);
        buf.len() - previous_len
    }

    fn colf_marshal_len(&self) -> ColferResult<usize> {
        let mut l = 1;
        if self.B {
            l += 1;
        }

        {
            let mut x = self.U32;
            if x >= 1 << 21 {
                l += 5;
            } else if x != 0 {
                l += 2;
                while x >= 0x80 {
                    x >>= 7;
                    l += 1;
                }
            }
        }

        {
            let mut x = self.U64;
            if x >= 1 << 49 {
                l += 9;
            } else if x != 0 {
                l += 2;
                while x >= 0x80 {
                    x >>= 7;
                    l += 1;
                }
            }
        }

        if self.I32 != 0 {
            let mut x = self.I32 as u32;
            if self.I32 < 0 {
                x = !x + 1;
            }
            l += 2;
            while x >= 0x80 {
                x >>= 7;
                l += 1;
            }
        }

        if self.I64 != 0 {
            l += 2;
            let mut x = self.I64 as u64;

            if self.I64 < 0 {
                x = !x + 1;
            }

            l += 2;

            for _ in 0..8 {
                if x <= 0x80 {
                    break;
                }

                x >>= 7;
                l += 1;
            }
        }

        if self.F32 != 0.0 {
            l += 5;
        }

        if self.F64 != 0.0 {
            l += 9;
        }

        if self.T != UNIX_EPOCH {
            let dur = self.T.duration_since(UNIX_EPOCH).unwrap();
            if (dur.as_secs() as u64) < 1 << 32 {
                l += 9;
            } else {
                l += 13;
            }
        }

        {
            let mut x = self.S.len();
            if x > 0 {
                if x > COLFER_SIZE_MAX {
                    return Err(ColferError::MaxSizeBreach {
                        field: "self::S",
                        overflow: x - COLFER_SIZE_MAX,
                    });
                }
                l += x + 2;
                while x >= 0x80 {
                    x >>= 7;
                    l += 1;
                }
            }
        }

        if let Some(ref v) = self.O {
            l += v.colf_marshal_len()? + 1;
        }

        {
            let mut x = self.Os.len();
            if x > 0 {
                if x > COLFER_LIST_MAX {
                    return Err(ColferError::MaxListBreach {
                        field: "self::Os",
                        overflow: x - COLFER_LIST_MAX,
                    });
                }

                l += 2;
                while x >= 0x80 {
                    x >>= 7;
                }

                for vo in self.Os.iter() {
                    if let Some(ref v) = vo {
                        l += v.colf_marshal_len()?;
                    } else {
                        l += 1;
                    }
                }

                if l > COLFER_SIZE_MAX {
                    return Err(ColferError::MaxSizeBreach {
                        field: "self::Os",
                        overflow: x - COLFER_SIZE_MAX,
                    });
                }
            }
        }

        {
            let mut x = self.Ss.len();
            if x > 0 {
                if x > COLFER_LIST_MAX {
                    return Err(ColferError::MaxListBreach {
                        field: "self::Ss",
                        overflow: x - COLFER_LIST_MAX,
                    });
                }

                l += 2;
                while x >= 0x80 {
                    x >>= 7;
                }

                for a in self.Ss.iter() {
                    let mut xs = a.len();
                    if xs > COLFER_SIZE_MAX {
                        return Err(ColferError::MaxSizeBreach {
                            field: "self::Ss",
                            overflow: xs - COLFER_SIZE_MAX,
                        });
                    }

                    l += xs + 1;
                    while xs >= 0x80 {
                        xs >>= 7;
                        l += 1;
                    }
                }

                if l >= COLFER_SIZE_MAX {
                    return Err(ColferError::MaxSizeBreach {
                        field: "self::Ss",
                        overflow: l - COLFER_SIZE_MAX,
                    });
                }
            }
        }

        {
            let mut x = self.As.len();
            if x > 0 {
                if x > COLFER_LIST_MAX {
                    return Err(ColferError::MaxListBreach {
                        field: "self::As",
                        overflow: x - COLFER_LIST_MAX,
                    });
                }

                l += 2;
                while x >= 0x80 {
                    x >>= 7;
                    l += 1;
                }

                for a in self.As.iter() {
                    let mut xs = a.len();
                    if xs > COLFER_SIZE_MAX {
                        return Err(ColferError::MaxSizeBreach {
                            field: "self::As",
                            overflow: xs - COLFER_SIZE_MAX,
                        });
                    }

                    l += xs + 1;
                    while xs >= 0x80 {
                        xs >>= 7;
                        l += 1;
                    }
                }

                if l >= COLFER_SIZE_MAX {
                    return Err(ColferError::MaxSizeBreach {
                        field: "self::Ss",
                        overflow: l - COLFER_SIZE_MAX,
                    });
                }
            }
        }

        if self.U8 > 0 {
            l += 2;
        }

        if self.U16 >= 1 << 8 {
            l += 3;
        } else if self.U16 != 0 {
            l += 2;
        }

        {
            let mut x = self.F32s.len();
            if x > 0 {
                if x > COLFER_LIST_MAX {
                    return Err(ColferError::MaxListBreach {
                        field: "self::F32s",
                        overflow: x - COLFER_LIST_MAX,
                    });
                }

                l += 2 + x * 4;
                while x >= 0x80 {
                    x >>= 7;
                    l += 1;
                }
            }
        }

        {
            let mut x = self.F64s.len();
            if x > 0 {
                if x > COLFER_LIST_MAX {
                    return Err(ColferError::MaxListBreach {
                        field: "self::F32s",
                        overflow: x - COLFER_LIST_MAX,
                    });
                }

                l += 2 + x * 8;
                while x >= 0x80 {
                    x >>= 7;
                    l += 1;
                }
            }
        }

        if l > COLFER_SIZE_MAX {
            return Err(ColferError::MaxSizeBreach {
                field: "self",
                overflow: l - COLFER_SIZE_MAX,
            });
        }

        Ok(l)
    }

    fn colf_unmarshal(&mut self, data: &'a [u8]) -> ColferResult<usize> {
        let len = data.len();
        if len == 0 {
            return Err(ColferError::UnexpectedEof);
        }
        let mut buf = ::std::io::Cursor::new(data);

        loop {
            let header = buf.get_u8();
            buf_guard!(buf);
            match header {
                0 => {
                    self.B = true;
                }
                1 => {
                    let mut x = buf.get_u32_be();
                    if x >= 0x80 {
                        x &= 0x7F;
                        let mut shift: u8 = 7;
                        loop {
                            let b = buf.get_u8() as u32;
                            buf_guard!(buf);

                            if b < 0x80 {
                                x |= b << shift;
                                break;
                            }
                            x |= (b & 0x7F) << shift;
                            shift += 7;
                        }
                    }
                    self.U32 = x;
                }
                129 => {
                    // 1 | 0x80
                    self.U32 = buf.get_u32_be();
                }
                2 => {
                    let mut x = buf.get_u64_be();
                    if x >= 0x80 {
                        x &= 0x7F;
                        let mut shift: u8 = 7;
                        loop {
                            let b = buf.get_u8() as u64;
                            buf_guard!(buf);

                            if b < 0x80 || shift == 56 {
                                x |= b << shift;
                                break;
                            }
                            x |= (b & 0x7F) << shift;
                            shift += 7;
                        }
                    }
                    self.U64 = x;
                }
                130 => {
                    // 2 | 0x80
                    self.U64 = buf.get_u64_be();
                }
                3 => {
                    let mut x = buf.get_u32_be();
                    if x >= 0x80 {
                        x &= 0x7F;
                        let mut shift: u8 = 7;
                        loop {
                            let b = buf.get_u8() as u32;
                            buf_guard!(buf);

                            if b < 0x80 {
                                x |= b << shift;
                                break;
                            }
                            x |= (b & 0x7F) << shift;
                            shift += 7;
                        }
                    }
                    self.I32 = x as i32;
                }
                131 => {
                    let mut x = buf.get_u32_be();
                    if x >= 0x80 {
                        x &= 0x7F;
                        let mut shift: u8 = 7;
                        loop {
                            let b = buf.get_u8() as u32;
                            buf_guard!(buf);

                            if b < 0x80 {
                                x |= b << shift;
                                break;
                            }
                            x |= (b & 0x7F) << shift;
                            shift += 7;
                        }
                    }
                    self.I32 = (!x + 1) as i32;
                }
                4 => {
                    let mut x = buf.get_u64_be();
                    if x >= 0x80 {
                        x &= 0x7F;
                        let mut shift: u8 = 7;
                        loop {
                            let b = buf.get_u8() as u64;
                            buf_guard!(buf);

                            if b < 0x80 || shift == 56 {
                                x |= b << shift;
                                break;
                            }
                            x |= (b & 0x7F) << shift;
                            shift += 7;
                        }
                    }
                    self.I64 = x as i64;
                }
                132 => {
                    let mut x = buf.get_u64_be();
                    if x >= 0x80 {
                        x &= 0x7F;
                        let mut shift: u8 = 7;
                        loop {
                            let b = buf.get_u8() as u64;
                            buf_guard!(buf);

                            if b < 0x80 || shift == 56 {
                                x |= b << shift;
                                break;
                            }
                            x |= (b & 0x7F) << shift;
                            shift += 7;
                        }
                    }
                    self.I64 = (!x + 1) as i64;
                }
                5 => {
                    self.F32 = f32::from_bits(buf.get_u32_be());
                }
                6 => {
                    self.F64 = f64::from_bits(buf.get_u64_be());
                }
                7 => {
                    let dur = Duration::new(buf.get_u32_be() as u64, buf.get_u32_be());
                    self.T = SystemTime::now() - dur;
                }
                135 => {
                    let dur = Duration::new(buf.get_u64_be(), buf.get_u32_be());
                    self.T = SystemTime::now() - dur;
                }
                8 => {
                    let mut x = buf.get_u8() as usize;
                    if x >= 0x80 {
                        x &= 0x7F;
                        let mut shift: u8 = 7;
                        loop {
                            let b = buf.get_u8() as usize;
                            buf_guard!(buf);

                            if b < 0x80 {
                                x |= b << shift;
                                break;
                            }
                            x |= (b & 0x7F) << shift;
                            shift += 7;
                        }
                    }

                    if x > COLFER_SIZE_MAX as usize {
                        return Err(ColferError::MaxSizeBreach {
                            field: "",
                            overflow: x - COLFER_LIST_MAX as usize,
                        });
                    }
                    buf_guard!(buf);

                    let start = buf.position() as usize;
                    self.S = ::std::str::from_utf8(&buf.get_ref()[start..x]).unwrap();
                    buf.set_position(x as u64 + 1);
                }
                9 => {
                    let mut x = buf.get_u8() as usize;
                    if x >= 0x80 {
                        x &= 0x7F;
                        let mut shift: u8 = 7;
                        loop {
                            let b = buf.get_u8() as usize;
                            buf_guard!(buf);

                            if b < 0x80 {
                                x |= b << shift;
                                break;
                            }
                            x |= (b & 0x7F) << shift;
                            shift += 7;
                        }
                    }

                    if x > COLFER_SIZE_MAX as usize {
                        return Err(ColferError::MaxSizeBreach {
                            field: "",
                            overflow: x - COLFER_LIST_MAX as usize,
                        });
                    }
                    buf_guard!(buf);

                    let start = buf.position() as usize;
                    let mut a = Vec::with_capacity(x - start);
                    a.copy_from_slice(&buf.get_ref()[start..x]);
                    self.A = a;
                }
                10 => {
                    let mut obj = Self::default();
                    let start = buf.position() as usize;
                    let n = obj.colf_unmarshal(&buf.get_ref()[start..])?;
                    self.O = Some(Box::new(obj));
                    buf.set_position(start as u64 + n as u64);
                    buf_guard!(buf);
                }
                11...17 => {
                    unimplemented!();
                }
                _ => {}
            }
        }
    }
}

//! Not for the faint of heart

use std::{fmt::Debug, io::Read};
use serde::Deserialize;

use super::{AnimXError, AnimXReader};

// This trait is kinda funny, but I didn't want to deal with passing around an ``impl Write<W>`` since ``dyn TrackTrait`` got quite angry about it
pub(crate) trait WriteBytes where Self: Debug {
    fn write(&self, write: &mut dyn FnMut(&[u8]));
}
pub(crate) trait ReadBytes where Self: Sized {
    fn read(reader: &mut AnimXReader<impl Read>) -> Result<Self, AnimXError>;
}

// These traits aren't great... oh well
#[allow(private_bounds)]
pub trait TrackTrait where Self: WriteBytes + Debug {}
pub(crate) trait KeyframeTrait where Self: WriteBytes + Debug {}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum TrackType {
    Raw,
    Discrete,
    Curve,
    Bezier,
}

impl TryFrom<u8> for TrackType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Raw),
            1 => Ok(Self::Discrete),
            2 => Ok(Self::Curve),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ValueType {
    Bool, Bool2, Bool3, Bool4,
    Byte, Ushort, Uint, Ulong,
    Sbyte, Short, Int, Long,
    Int2, Int3, Int4,
    Uint2, Uint3, Uint4,
    Long2, Long3, Long4,
    Float, Float2, Float3, Float4,
    FloatQ, Float2x2, Float3x3, Float4x4,
    Double, Double2, Double3, Double4,
    DoubleQ, Double2x2, Double3x3, Double4x4,
    Color, Color32,
    #[serde(rename = "string")]
    OptString,
}

impl TryFrom<u8> for ValueType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        metamatch::metamatch!(match value {
            #[expand(for (I,T) in enumerate([
                Byte, Ushort, Ulong, Sbyte, Short,
                Bool, Bool2, Bool3, Bool4,
                Int, Int2, Int3, Int4,
                Uint, Uint2, Uint3, Uint4,
                Long, Long2, Long3, Long4,
                Float, Float2, Float3, Float4,
                FloatQ, Float2x2, Float3x3, Float4x4,
                Double, Double2, Double3, Double4,
                DoubleQ, Double2x2, Double3x3, Double4x4,
                Color, Color32, OptString,
            ]))]
            I => Ok(Self::T),
            _ => Err(()),
        })
    }
}

impl WriteBytes for ValueType {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        write(&[*self as u8]);
    }
}

impl<T> WriteBytes for Option<T> where T: WriteBytes + Default {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        self.as_ref().unwrap_or(&Default::default()).write(write);
    }
}

/// Little hack to make writing structure byte lengths as varints easy
type VarInt = usize;

/// Little hack to make writing structure byte lengths as varints easy
impl WriteBytes for VarInt {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        let mut value = *self;
        while value > 127 {
            ((value & 127 | 128) as u8).write(write);
            value >>= 7;
        }

        (value as u8).write(write);
    }
}

impl WriteBytes for String {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        let bytes = self.as_bytes();
        bytes.len().write(write);
        write(bytes);
    }
}

/// Since header strings don't write a nullable byte (even though they're optional!) this wrapper is used as the type for "string" tracks

// I honestly don't know why this is here, why not just denote empty strings with a size of 0? it would take up less space in the AnimX format
#[derive(Deserialize, Debug, Default)]
pub struct OptString(pub String);

impl WriteBytes for OptString {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        let bytes = self.0.as_bytes();
        if bytes.len() == 0 {
            write(&[0x00]);
            return;
        }
        write(&[0x01]);
        bytes.len().write(write);
        write(bytes);
    }
}

impl ReadBytes for OptString {
    fn read(reader: &mut AnimXReader<impl Read>) -> Result<Self, AnimXError> {
        reader.read_nullable_string().map(|v| OptString(v.unwrap_or_default()))
    }
}

metamatch::quote! {
    [<for (name, internal) in [(Color, f32), (Color32, u8)]>]
        #[derive(Debug, Deserialize, Clone, Copy)]
        pub struct [<ident(str(name))>]  {
            [<for field in [r,g,b,a]>]
                [<ident(str(field))>]: [<ident(str(internal))>],
            [</for>]
        }
        
        impl WriteBytes for [<ident(str(name))>] {
            fn write(&self, write: &mut dyn FnMut(&[u8])) {
                [<for field in [r,g,b,a]>]
                    self.[<ident(str(field))>].write(write);
                [</for>]
            }
        }

        impl ReadBytes for [<ident(str(name))>]  {
            fn read(reader: &mut AnimXReader<impl Read>) -> Result<Self, AnimXError> {
                Ok(Self{
                [<for field in [r,g,b,a]>]
                    [<ident(str(field))>]: [<ident(str(internal))>]::read(reader)?,
                [</for>]
                })
            }
        }
    [</for>]
}

pub type Byte = u8;
pub type Ushort = u16;
pub type Ulong = u64;
pub type Sbyte = i8;
pub type Short = i16;
pub type FloatQ = Float4;
pub type DoubleQ = Double4;
pub type Float2x2 = [[Float; 2]; 2];
pub type Float3x3 = [[Float; 3]; 3];
pub type Float4x4 = [[Float; 4]; 4];
pub type Double2x2 = [[Double; 2]; 2];
pub type Double3x3 = [[Double; 3]; 3];
pub type Double4x4 = [[Double; 4]; 4];

impl WriteBytes for Bool {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        write(&[if *self {1} else {0}]);
    }
}

impl WriteBytes for Bool2 {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        write(&[if self.x {1} else {0} | if self.y {2} else {0}]);
    }
}

impl WriteBytes for Bool3 {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        write(&[if self.x {1} else {0} | if self.y {2} else {0} | if self.z {4} else {0}]);
    }
}

impl WriteBytes for Bool4 {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        write(&[if self.x {1} else {0} | if self.y {2} else {0} | if self.z {4} else {0} | if self.w {8} else {0}]);
    }
}

impl ReadBytes for Bool {
    fn read(reader: &mut AnimXReader<impl Read>) -> Result<Self, AnimXError> {
        Ok(reader.read_bool()?)
    }
}

metamatch::quote! {
    [<for I in 2..5>]
        impl ReadBytes for [<ident("Bool" + str(I))>] {
            fn read(reader: &mut AnimXReader<impl Read>) -> Result<Self, AnimXError> {
                let byte = reader.read_u8()?;
                Ok(Self{
                    [<for index in 0..I>]
                        [<let field = [x,y,z,w][index]>]
                        [<ident(str(field))>]: byte >> [<index>] & 1 == 1,
                    [</for>]
                })
            }
        }
    [</for>]
}

metamatch::quote! {
    [<for (name, size) in [(Byte, 1), (Sbyte, 1), (Ushort, 2), (Ulong, 8), (Short, 2), (Int, 4), (Long, 8), (Uint, 4), (Float, 4), (Double, 8)]>]
        impl WriteBytes for [<ident(str(name))>] {
            fn write(&self, write: &mut dyn FnMut(&[u8])) {
                write(&self.to_le_bytes());
            }
        }

        impl ReadBytes for [<ident(str(name))>]  {
            fn read(reader: &mut AnimXReader<impl Read>) -> Result<Self, AnimXError> {
                let mut buf = [0u8; [<(size)>]];
                reader.read_into(&mut buf)?;
                Ok(Self::from_le_bytes(buf))
            }
        }
    [</for>]
}

metamatch::quote! {
    [<for type in [Float, Double]>]
        [<for size in 2..5>]
            [<let name = str(type) + str(size) + "x" + str(size)>]
            impl WriteBytes for [<ident(str(name))>] {
                fn write(&self, write: &mut dyn FnMut(&[u8])) {
                    self.iter().for_each(|i| i.iter().for_each(|i| i.write(write)))
                }
            }

            impl ReadBytes for [<ident(str(name))>] {
                fn read(reader: &mut AnimXReader<impl Read>) -> Result<Self, AnimXError> {
                    Ok([
                        [<for a in 0..size>]
                        [
                            [<for b in 0..size>]
                                [<ident(str(type))>]::read(reader)?,
                            [</for>]
                        ],
                        [</for>]
                    ])
                }
            }
        [</for>]
    [</for>]
}

metamatch::quote! {
    [<for (name, internal, derive) in [(Bool, bool, false), (Int, i32, true), (Long, i64, true), (Uint, u32, true), (Float, f32, true), (Double, f64, true)]>]
        pub type [<ident(str(name))>] = [<ident(str(internal))>];

        [<for range in 2..5>]
            #[derive(Debug, Deserialize, Clone, Copy)]
            pub struct [<ident(str(name) + str(range))>] {
                [<for field in 0..range>]
                    [<let field_name = [x,y,z,w][field]>]
                    pub [<ident(str(field_name))>]: [<ident(str(internal))>],
                [</for>]
            }

            [<if derive>]
            impl WriteBytes for [<ident(str(name) + str(range))>] {
                fn write(&self, write: &mut dyn FnMut(&[u8])) {
                    [<for field in 0..range>]
                        [<let field_name = [x,y,z,w][field]>]
                        write( &self.[<ident(str(field_name))>].to_le_bytes() );
                    [</for>]
                }
            }

            impl ReadBytes for [<ident(str(name) + str(range))>]  {
                fn read(reader: &mut AnimXReader<impl Read>) -> Result<Self, AnimXError> {
                    Ok(Self {
                        [<for field in 0..range>]
                            [<let field_name = [x,y,z,w][field]>]
                            [<ident(str(field_name))>]: [<ident(str(name))>]::read(reader)?,
                        [</for>]
                    })
                }
            }
            [</if>]
        [</for>]
    [</for>]
}

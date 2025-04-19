//! # Animation data (AnimJ & AnimX)

pub mod types;
use types::*;

use std::{fmt::Debug, io::{BufWriter, Write}};
use serde::{de::{Error, IgnoredAny, Visitor}, Deserialize, Deserializer};

/// The overarching type for animations
/// 
/// This type implements ``serde::Deserialize`` and is meant to be deserealized from an AnimJ (JSON) structure
/// 
/// There is also a function for writing an AnimX stream (Binary)
/// 
/// Deserealizing AnimX is currently **not** supported
#[allow(private_interfaces)]
#[derive(Debug, Default)]
pub struct Animation {
    pub name: Option<String>,
    pub global_duration: Option<f32>,
    pub tracks: Vec<Box<dyn TrackTrait>>,
}

impl Animation {
    /// Function for writing data as an AnimX stream
    /// 
    /// Compression is not yet supported.
    /// 
    /// ```
    /// use resonite_core::animation::Animation;
    /// 
    /// let anim: Animation = serde_json::from_str(/* AnimJ */)?;
    /// let mut buf = Vec::new();
    /// anim.write_animx(&mut buf);
    /// ```
    /// 
    pub fn write_animx(&self, buf: impl Write) {
        let mut writer = BufWriter::new(buf);
        let mut write = |bytes: &[u8]| { writer.write(bytes).unwrap(); };

        self.write_contents(&mut write);
    }

    fn write_contents(&self, write: &mut dyn FnMut(&[u8])) {
        "AnimX".to_owned().write(write);    // "AnimX" magic header
        01u32.write(write);                 // Version 01 (wiki says this is supposed to be a byte, but it's an Int / i32)
        self.tracks.len().write(write);     // Length (wiki says this is supposed to be a 7bit integer, but this is actually a varint)
        self.global_duration.write(write);  // Length of animation in seconds
        self.name.write(write);             // Name of animation
        write(&[0x00,]);                    // Encoding flag (just none for now)
        for track in &self.tracks {
            track.write(write);             // Tracks
        }
    }
}

impl<'de> Deserialize<'de> for Animation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct AnimVisitor;

        impl<'de> Visitor<'de> for AnimVisitor {
            type Value = Animation;
        
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map with a tracks list")
            }
            
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut output = Animation::default();
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "name" => {
                            let name: String = map.next_value()?;
                            output.name = Some(name);
                        },
                        "globalDuration" => {
                            let f: f32 = map.next_value()?;
                            output.global_duration = Some(f);
                        },
                        "tracks" => {
                            let v: serde_json::Value = map.next_value()?;
                            let tracks = v.as_array().ok_or(Error::custom("incorrect field type for \"tracks\", expected 'Value::Array'"))?;
                            let tracks = tracks.iter().map(|v| {
                                let v = v.clone();
                                let info: TrackInfo = serde_json::from_value(v.clone())?;

                                // This technically makes Curve keyframes on String values possible...
                                let track = metamatch::metamatch!(match info.track_type {
                                    #[expand(for (T,X) in [
                                        (Raw, RawData),
                                        (Discrete, DiscreteData),
                                        (Curve, CurveData),
                                    ])]
                                    TrackType::T => {
                                        metamatch::metamatch!(match info.value_type {
                                            #[expand(for V in [
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
                                            ])]
                                            ValueType::V => serde_json::from_value::<Box<Track<X<V>>>>(v)? as Box<dyn TrackTrait>,
                                        })
                                    },
                                    TrackType::Bezier => todo!(),
                                });
                                Ok(track)
                            }).map(|r| r.map_err(|e: serde_json::Error| Error::custom(e)));
                            for track in tracks {
                                output.tracks.push(track?);
                            }
                        },
                        _ => {
                            let _: IgnoredAny = map.next_value()?;
                        },
                    }
                }

                Ok(output)
            }
        }

        deserializer.deserialize_any(AnimVisitor)
    }
}

#[derive(Debug, Deserialize)]
struct TrackInfo where {
    #[serde(rename = "trackType")]
    pub track_type: TrackType,
    #[serde(rename = "valueType")]
    pub value_type: ValueType,
}

#[allow(private_bounds)]
#[derive(Debug, Deserialize)]
pub struct Track<T> where T: KeyframeTrait {
    #[serde(rename = "trackType")]
    pub track_type: TrackType,
    #[serde(rename = "valueType")]
    pub value_type: ValueType,
    pub data: T,
}

impl<T> WriteBytes for Track<T> where T: KeyframeTrait {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        write(&[self.track_type as u8, self.value_type as u8]);
        self.data.write(write);
    }
}

impl<T> TrackTrait for Track<T> where T: KeyframeTrait {}

#[allow(private_bounds)]
#[derive(Debug, Deserialize)]
pub struct RawData<T> where T: WriteBytes + Debug {
    pub node: Option<String>,
    pub property: Option<String>,
    pub interval: Option<f32>,
    pub keyframes: Vec<T>,
}

impl<T> WriteBytes for RawData<T> where T: WriteBytes + Debug {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        self.node.write(write);
        self.property.write(write);
        self.keyframes.len().write(write);
        self.interval.write(write);
        for keyframe in &self.keyframes {
            keyframe.write(write);
        }
    }
}

impl<T> KeyframeTrait for RawData<T> where T: WriteBytes + Debug {}

#[allow(private_bounds)]
#[derive(Debug, Deserialize)]
pub struct DiscreteData<T> where T: WriteBytes + Debug {
    pub node: Option<String>,
    pub property: Option<String>,
    pub keyframes: Vec<DiscreteKeyframe<T>>,
}

impl<T> WriteBytes for DiscreteData<T> where T: WriteBytes + Debug {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        self.node.write(write);
        self.property.write(write);
        self.keyframes.len().write(write);
        for keyframe in &self.keyframes {
            keyframe.write(write);
        }
    }
}

impl<T> KeyframeTrait for DiscreteData<T> where T: WriteBytes + Debug {}

#[allow(private_bounds)]
#[derive(Debug, Deserialize)]
pub struct DiscreteKeyframe<T> where T: WriteBytes + Debug {
    pub time: f32,
    pub value: T,
}

impl<T> WriteBytes for DiscreteKeyframe<T> where T: WriteBytes + Debug {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        self.time.write(write);
        self.value.write(write);
    }
}

#[allow(private_bounds)]
#[derive(Debug, Deserialize)]
pub struct CurveData<T> where T: WriteBytes + Debug {
    pub node: Option<String>,
    pub property: Option<String>,
    pub keyframes: Vec<CurveKeyframe<T>>,
}

impl<T> WriteBytes for CurveData<T> where T: WriteBytes + Debug {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        let interpolation = self.keyframes.first().map(|k| k.interpolation).unwrap_or(Interpolation::Hold);
        let mut info = 0x1;
        for keyframe in &self.keyframes {
            if keyframe.interpolation != interpolation {
                info |= 0x1;
            }
            info |= keyframe.interpolation as u8 & 0x2;
        }

        self.node.write(write);
        self.property.write(write);
        self.keyframes.len().write(write);
        write(&[info]);

        if info & 0x1 == 0x1 {
            for keyframe in &self.keyframes {
                (keyframe.interpolation as u8).write(write);
            }
        } else {
            (interpolation as u8).write(write);
        }

        for keyframe in &self.keyframes {
            keyframe.write(write);
        }

        if info & 0x2 == 0x2 {
            for keyframe in &self.keyframes {
                keyframe.left_tangent.as_ref().expect("interpolation mode was tangent or bezier, but leftTangent wasn't present").write(write);
                keyframe.right_tangent.as_ref().expect("interpolation mode was tangent or bezier, but rightTangent wasn't present").write(write);
            }
        }
    }
}

impl<T> KeyframeTrait for CurveData<T> where T: WriteBytes + Debug {}

#[allow(private_bounds)]
#[derive(Debug, Deserialize)]
pub struct CurveKeyframe<T> where T: WriteBytes + Debug {
    pub time: f32,
    pub value: T,
    pub interpolation: Interpolation,

    /// I think the types for ``left_tangent`` & ``right_tangent`` are incorrect but I'm not sure what they should be...\
    /// Maybe they're supposed to be ``(f32, T)`` pairs?

    #[serde(rename = "leftTangent")]
    pub left_tangent: Option<T>,
    #[serde(rename = "rightTangent")]
    pub right_tangent: Option<T>,
}

impl<T> WriteBytes for CurveKeyframe<T> where T: WriteBytes + Debug {
    fn write(&self, write: &mut dyn FnMut(&[u8])) {
        self.time.write(write);
        self.value.write(write);
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone, Copy)]
pub enum Interpolation {
    Hold,
    Linear,
    Tangent,
    CubicBezier,
}

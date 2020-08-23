use anyhow::{anyhow, Result};
use num_traits::cast::FromPrimitive;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub enum FieldValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Double(f64),
    String(String),
    Time(Time),
    Array(Vec<FieldValue>),
    Map(HashMap<String, FieldValue>),
}

impl FieldValue {
    pub fn to_primitive<T>(self) -> Result<T>
    where
        T: Converter,
    {
        T::to_primitive(self)
    }
}

#[derive(Clone, Debug)]
pub struct Time {
    pub seconds: i64,
    pub nanos: i32,
}

pub trait Mapper {
    fn to_map(&self) -> HashMap<String, FieldValue>;
    fn from_map(map: HashMap<String, FieldValue>) -> Result<Self>
    where
        Self: std::marker::Sized;
}

pub trait Converter: Sized {
    fn to_field_value(&self) -> FieldValue;
    fn to_primitive(fv: FieldValue) -> Result<Self>;
}

impl Converter for String {
    fn to_field_value(&self) -> FieldValue {
        FieldValue::String(self.to_string())
    }
    fn to_primitive(fv: FieldValue) -> Result<Self> {
        match fv {
            FieldValue::String(value) => Ok(value),
            _ => Err(anyhow!("invalid type: String")),
        }
    }
}

impl Converter for char {
    fn to_field_value(&self) -> FieldValue {
        FieldValue::String(self.to_string())
    }
    fn to_primitive(fv: FieldValue) -> Result<Self> {
        match fv {
            FieldValue::String(value) => {
                let chars: Vec<char> = value.chars().collect();
                if chars.len() != 1 {
                    return Err(anyhow!("invalid type: char"));
                }
                return Ok(chars[0]);
            }
            _ => Err(anyhow!("invalid type: char")),
        }
    }
}

impl Converter for bool {
    fn to_field_value(&self) -> FieldValue {
        FieldValue::Boolean(*self)
    }

    fn to_primitive(fv: FieldValue) -> Result<Self> {
        match fv {
            FieldValue::Boolean(value) => Ok(value),
            _ => Err(anyhow!("invalid type: bool")),
        }
    }
}

impl Converter for Time {
    fn to_field_value(&self) -> FieldValue {
        FieldValue::Time(self.clone())
    }

    fn to_primitive(fv: FieldValue) -> Result<Self> {
        match fv {
            FieldValue::Time(value) => Ok(value),
            _ => Err(anyhow!("invalid type: Time")),
        }
    }
}

macro_rules! integer_impls {
    ($($type:ty)+) => {
        $(
            impl Converter for $type {
                #[inline]
                fn to_field_value(&self) -> FieldValue {
                    FieldValue::Integer(*self as i64)
                }

                #[inline]
                fn to_primitive(fv: FieldValue) ->Result<Self>{
                    match fv {
                        FieldValue::Integer(value)=>{
                            if let Ok(value) = <$type>::try_from(value) {
                                return Ok(value);
                            }
                            return Err(anyhow!("invalid type: {}",stringify!($type)))
                        },
                        _=> Err(anyhow!("invalid type: {}",stringify!($type))),
                    }
                }
            }
        )+
    }
}

integer_impls! {
    i8 i16 i32 i64 isize u8 u16 u32
}

impl Converter for f32 {
    fn to_field_value(&self) -> FieldValue {
        FieldValue::Double(*self as f64)
    }
    fn to_primitive(fv: FieldValue) -> Result<Self> {
        match fv {
            FieldValue::Double(value) => {
                if let Some(value) = f32::from_f64(value) {
                    return Ok(value);
                }
                return Err(anyhow!("invalid type: f32"));
            }
            _ => Err(anyhow!("invalid type: f32")),
        }
    }
}

impl Converter for f64 {
    fn to_field_value(&self) -> FieldValue {
        FieldValue::Double(*self)
    }
    fn to_primitive(fv: FieldValue) -> Result<Self> {
        match fv {
            FieldValue::Double(value) => Ok(value),
            _ => Err(anyhow!("invalid type: f64")),
        }
    }
}

impl<T> Converter for Option<T>
where
    T: Converter,
{
    fn to_field_value(&self) -> FieldValue {
        match self {
            Some(some) => some.to_field_value(),
            None => FieldValue::Null,
        }
    }

    fn to_primitive(fv: FieldValue) -> Result<Self> {
        match fv {
            FieldValue::Null => Ok(None),
            _ => Ok(Some(T::to_primitive(fv)?)),
        }
    }
}

impl<T> Converter for Vec<T>
where
    T: Converter,
{
    fn to_field_value(&self) -> FieldValue {
        FieldValue::Array(self.iter().map(|v| v.to_field_value()).collect())
    }

    fn to_primitive(fv: FieldValue) -> Result<Self> {
        match fv {
            FieldValue::Array(value) => value
                .into_iter()
                .map(|v| T::to_primitive(v))
                .collect::<Result<Vec<T>>>(),
            _ => Err(anyhow!("invalid type: Vec<T>")),
        }
    }
}

impl<K, V> Converter for HashMap<K, V>
where
    K: ToString + From<String> + std::cmp::Eq + std::hash::Hash,
    V: Converter,
{
    fn to_field_value(&self) -> FieldValue {
        FieldValue::Map(
            self.iter()
                .map(|(key, value)| (key.to_string(), value.to_field_value()))
                .collect(),
        )
    }

    fn to_primitive(fv: FieldValue) -> Result<Self> {
        match fv {
            FieldValue::Map(value) => {
                let mut result = HashMap::with_capacity(value.len());
                for (k, v) in value {
                    if let Ok(k) = K::try_from(k) {
                        result.insert(k, V::to_primitive(v)?);
                    } else {
                        return Err(anyhow!("invalid type: HashMap<K, V>"));
                    }
                }
                return Ok(result);
            }
            _ => Err(anyhow!("invalid type: HashMap<K, V>")),
        }
    }
}

impl<T> Converter for T
where
    T: Mapper,
{
    fn to_field_value(&self) -> FieldValue {
        FieldValue::Map(self.to_map())
    }

    fn to_primitive(fv: FieldValue) -> Result<Self> {
        match fv {
            FieldValue::Map(value) => Ok(T::from_map(value)?),
            _ => Err(anyhow!("invalid type: Mapper")),
        }
    }
}

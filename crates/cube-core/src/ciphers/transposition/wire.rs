//! Concrete integer DTO transport shared by these bounded profiles. Accept the
//! canonical decimal strings advertised by the catalog without rounding in JS.
use serde::{Deserialize, Deserializer, de::Error};
use std::{fmt::Display, str::FromStr};
#[derive(Deserialize)]
#[serde(untagged)]
enum Integer {
    Unsigned(u64),
    Signed(i64),
    Decimal(String),
}
fn convert<T, E: Error>(value: Integer) -> Result<T, E>
where
    T: TryFrom<u64> + TryFrom<i64> + FromStr + Display,
{
    match value {
        Integer::Unsigned(v) => {
            T::try_from(v).map_err(|_| E::custom("integer outside native range"))
        }
        Integer::Signed(v) => T::try_from(v).map_err(|_| E::custom("integer outside native range")),
        Integer::Decimal(s) => {
            if s.len() > 20 {
                return Err(E::custom("integer outside native range"));
            }
            let value = s
                .parse::<T>()
                .map_err(|_| E::custom("canonical native decimal integer required"))?;
            if value.to_string() != s {
                return Err(E::custom("canonical native decimal integer required"));
            }
            Ok(value)
        }
    }
}
pub(crate) fn integer<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: TryFrom<u64> + TryFrom<i64> + FromStr + Display,
{
    convert(Integer::deserialize(d)?)
}
pub(crate) fn optional<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: TryFrom<u64> + TryFrom<i64> + FromStr + Display,
{
    Option::<Integer>::deserialize(d)?.map(convert).transpose()
}
#[derive(Deserialize)]
struct Index(#[serde(deserialize_with = "integer")] usize);
pub(crate) fn indices<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<usize>, D::Error> {
    Ok(Vec::<Index>::deserialize(d)?
        .into_iter()
        .map(|v| v.0)
        .collect())
}

//! Strict digit-preserving binary-coded decimal profiles.
use crate::{
    errors::OperationError,
    formal_work::{invalid, parameter, resource},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_DIGITS: usize = 4096;

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    UnsignedPacked,
    UnsignedUnpacked,
    SignedPacked,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub profile: Profile,
    #[schemars(range(min = 1, max = 4096))]
    pub digits: usize,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Decode {
    pub profile: Profile,
    #[schemars(range(min = 1, max = 4096))]
    pub digits: usize,
    #[serde(default)]
    pub allow_unsigned_sign: bool,
}

#[derive(Serialize, JsonSchema)]
pub struct Decimal {
    pub digits: String,
    pub negative: bool,
}

impl Layout {
    fn size(self) -> Result<usize, OperationError> {
        if self.digits == 0 {
            return Err(parameter("BCD digit width must be positive").parameter("digits"));
        }
        if self.digits > MAX_DIGITS {
            return Err(resource("BCD digit width exceeds 4096"));
        }
        Ok(match self.profile {
            Profile::UnsignedPacked => self.digits.div_ceil(2),
            Profile::UnsignedUnpacked => self.digits,
            Profile::SignedPacked => (self.digits + 1).div_ceil(2),
        })
    }
}

/// Encode exact ASCII digits at the declared width, normalizing negative zero.
pub fn encode(digits: &str, negative: bool, layout: Layout) -> Result<Vec<u8>, OperationError> {
    let size = layout.size()?;
    if digits.len() > MAX_DIGITS {
        return Err(resource("BCD input exceeds 4096 digits"));
    }
    if digits.len() != layout.digits || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid(
            "BCD requires exactly the declared number of ASCII digits",
        ));
    }
    let signed = matches!(layout.profile, Profile::SignedPacked);
    if negative && !signed {
        return Err(invalid("unsigned BCD does not accept a negative sign"));
    }
    let digits = digits.bytes().map(|b| b - b'0');
    if matches!(layout.profile, Profile::UnsignedUnpacked) {
        return Ok(digits.collect());
    }
    let mut output = vec![0; size];
    let start = size * 2 - layout.digits - usize::from(signed);
    let mut zero = true;
    for (i, digit) in digits.enumerate() {
        zero &= digit == 0;
        let slot = i + start;
        output[slot / 2] |= digit << (4 * (1 - slot % 2));
    }
    if signed {
        output[size - 1] |= if negative && !zero { 0xd } else { 0xc };
    }
    Ok(output)
}

/// Decode a complete BCD value, preserving leading zeroes.
pub fn decode(data: &[u8], options: Decode) -> Result<Decimal, OperationError> {
    let layout = Layout {
        profile: options.profile,
        digits: options.digits,
    };
    let size = layout.size()?;
    if data.len() > 16 * 1024 {
        return Err(resource("BCD input exceeds 16 KiB"));
    }
    if data.len() != size {
        return Err(invalid("BCD byte length does not match digit width"));
    }
    let signed = matches!(layout.profile, Profile::SignedPacked);
    if options.allow_unsigned_sign && !signed {
        return Err(parameter("unsigned sign opt-in requires signed packed BCD")
            .parameter("allow_unsigned_sign"));
    }
    let mut digits = String::with_capacity(layout.digits);
    let mut push = |digit| {
        if digit > 9 {
            return Err(invalid("BCD digit nibble exceeds nine"));
        }
        digits.push(char::from(b'0' + digit));
        Ok(())
    };
    if matches!(layout.profile, Profile::UnsignedUnpacked) {
        for &digit in data {
            push(digit)?;
        }
    } else {
        let start = size * 2 - layout.digits - usize::from(signed);
        if start != 0 && data[0] >> 4 != 0 {
            return Err(invalid("BCD leading padding nibble must be zero"));
        }
        for slot in start..start + layout.digits {
            push((data[slot / 2] >> (4 * (1 - slot % 2))) & 0xf)?;
        }
    }
    let negative = signed
        && match data[size - 1] & 0xf {
            0xc => false,
            0xd => digits.bytes().any(|b| b != b'0'),
            0xf if options.allow_unsigned_sign => false,
            _ => return Err(invalid("invalid signed BCD sign nibble")),
        };
    Ok(Decimal { digits, negative })
}

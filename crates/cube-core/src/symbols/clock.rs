//! Exact single-reading clock-to-direction convention, separate from flag Semaphore.
use super::semaphore;
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
const TURN: i64 = 360000;
pub const TOLERANCE: i64 = 1;
fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
}
#[derive(Debug, Serialize)]
pub struct Hand {
    pub normalized_input_millidegrees: u32,
    pub bearing_millidegrees: u32,
    pub directions: Vec<&'static str>,
    pub ambiguous: bool,
}
#[derive(Debug, Serialize)]
pub struct Clock {
    pub hour: Hand,
    pub minute: Hand,
    pub zero_axis: String,
    pub clockwise: bool,
    pub tolerance_millidegrees: i64,
    pub decoded: Option<String>,
    pub interpretation: &'static str,
}
fn hand(angle: i64, offset: i64, clockwise: bool) -> Hand {
    let normalized = angle.rem_euclid(TURN);
    let bearing = (offset + if clockwise { normalized } else { -normalized }).rem_euclid(TURN);
    let sector = (bearing / 45000) as usize;
    let ambiguous = (bearing % 45000 - 22500).abs() <= TOLERANCE;
    let directions = if ambiguous {
        vec![
            semaphore::DIRECTIONS[sector],
            semaphore::DIRECTIONS[(sector + 1) % 8],
        ]
    } else {
        vec![semaphore::DIRECTIONS[((bearing + 22500) / 45000) as usize % 8]]
    };
    Hand {
        normalized_input_millidegrees: normalized as u32,
        bearing_millidegrees: bearing as u32,
        directions,
        ambiguous,
    }
}
fn time_angles(time: &str) -> Result<(i64, i64), OperationError> {
    if time.len() > 5 {
        return Err(invalid("time must be H:MM or HH:MM, without seconds"));
    }
    let (h, m) = time
        .split_once(':')
        .ok_or_else(|| invalid("time requires H:MM"))?;
    if !(1..=2).contains(&h.len())
        || m.len() != 2
        || !h.bytes().chain(m.bytes()).all(|b| b.is_ascii_digit())
    {
        return Err(invalid(
            "time requires ASCII H:MM, with no signs or whitespace",
        ));
    }
    let h = h.parse::<u8>().map_err(|_| invalid("invalid clock hour"))?;
    let m = m
        .parse::<u8>()
        .map_err(|_| invalid("invalid clock minute"))?;
    if h > 12 || m > 59 {
        return Err(invalid("12-hour time requires hour 0..12 and minute 0..59"));
    }
    Ok((
        i64::from(h % 12) * 30000 + i64::from(m) * 500,
        i64::from(m) * 6000,
    ))
}
/// Map exactly one time (H:MM, hours 0..12; 0=12) or two labeled signed i64
/// millidegree angles. No floats, fractional minutes or seconds. Time uses
/// continuous hour motion (0.5 degrees/minute). Clockwise is default; N/E/S/W
/// selects the zero axis. Bearings are receiver-facing, north=0 clockwise.
/// Within one millidegree of a 22.5-degree sector boundary, return both choices.
/// Optional decoding maps hour->sender LH, minute->sender RH and rejects ambiguous
/// or unsupported pairs. It reuses the letter-only Semaphore table, never guesses.
/// One reading per call, constant work/storage and output below 1 KiB.
pub fn inspect(
    time: Option<&str>,
    hour_millidegrees: Option<i64>,
    minute_millidegrees: Option<i64>,
    zero_axis: &str,
    clockwise: bool,
    decode: bool,
) -> Result<Clock, OperationError> {
    let result = (|| {
        let offset = match zero_axis {
            "N" => 0,
            "E" => 90000,
            "S" => 180000,
            "W" => 270000,
            _ => return Err(invalid("zero_axis must be N, E, S or W").parameter("zero_axis")),
        };
        let (hour, minute) = match (time, hour_millidegrees, minute_millidegrees) {
            (Some(t), None, None) => time_angles(t)?,
            (None, Some(h), Some(m)) => (h, m),
            _ => {
                return Err(invalid(
                    "select exactly one time or both labeled millidegree angles",
                ));
            }
        };
        let hour = hand(hour, offset, clockwise);
        let minute = hand(minute, offset, clockwise);
        let decoded = if decode {
            if hour.ambiguous || minute.ambiguous {
                return Err(invalid("ambiguous hand sectors cannot be decoded").parameter("decode"));
            }
            Some(semaphore::decode_pair(hour.directions[0], minute.directions[0])?.to_string())
        } else {
            None
        };
        Ok(Clock {
            hour,
            minute,
            zero_axis: zero_axis.into(),
            clockwise,
            tolerance_millidegrees: TOLERANCE,
            decoded,
            interpretation: "caller_selected_clock_convention_hour_LH_minute_RH",
        })
    })();
    result.map_err(|e: OperationError| e.operation("symbol.clock.directions"))
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "symbol.clock.directions",
    category: "symbols",
    description: "Map one exact H:MM reading or two signed millidegree angles to eight receiver-facing sectors; continuous hour motion, explicit orientation and +/-1 millidegree boundary ambiguity. Optional letter-only Semaphore composition is a caller convention.",
    input_type: "clock.reading",
    output_type: "clock.directions",
    parameters: &[
        Parameter {
            name: "time",
            description: "H:MM or HH:MM, hour 0..12 (0=12), minute 0..59; mutually exclusive with angles; no seconds.",
            required: false,
            default: None,
            constraints: ParameterType::Text { choices: &[] },
        },
        Parameter {
            name: "hour_millidegrees",
            description: "Exact signed hour-hand angle; requires minute_millidegrees, conflicts with time.",
            required: false,
            default: None,
            constraints: ParameterType::Integer {
                min: i64::MIN,
                max: i64::MAX,
            },
        },
        Parameter {
            name: "minute_millidegrees",
            description: "Exact signed minute-hand angle, normalized modulo 360000.",
            required: false,
            default: None,
            constraints: ParameterType::Integer {
                min: i64::MIN,
                max: i64::MAX,
            },
        },
        Parameter {
            name: "zero_axis",
            description: "Receiver-facing zero bearing.",
            required: false,
            default: Some(ParameterValue::Text("N")),
            constraints: ParameterType::Text {
                choices: &["N", "E", "S", "W"],
            },
        },
        Parameter {
            name: "clockwise",
            description: "Positive angles follow clockwise bearings; false reverses the sense.",
            required: false,
            default: Some(ParameterValue::Boolean(true)),
            constraints: ParameterType::Boolean,
        },
        Parameter {
            name: "decode",
            description: "Explicit hour=sender LH / minute=sender RH Semaphore composition; ambiguity or unsupported pairs fail.",
            required: false,
            default: Some(ParameterValue::Boolean(false)),
            constraints: ParameterType::Boolean,
        },
    ],
    deterministic: true,
    reversible: Some(false),
    command: "number clock",
    example: "cube number clock --time 3:00 --decode",
}];

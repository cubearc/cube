//! Independent known answers; these fixtures call the actual typed Core functions.
use super::*;
use crate::operations::contracts::Fixture;
use serde_json::{Value, json};
fn verified_key_text(
    op: &'static str,
    input: &str,
    parameters: Value,
    actual: String,
    expected: &str,
) -> Result<Fixture, OperationError> {
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            "published key-profile answer mismatch",
        )
        .operation(op));
    }
    Ok(Fixture {
        inputs: json!({"text":input}),
        parameters,
        outputs: json!({"text":actual}),
    })
}

pub(crate) fn interrupted_key_encode() -> Result<Fixture, OperationError> {
    let p = key_profiles::InterruptedKey {
        key: "ORANGE".into(),
        resets: vec![4, 10, 12, 15, 19, 22, 23, 24, 29, 30, 32, 35],
    };
    let text = "THISCIPHERCANBEUSEDWITHANYOFTHEPERIODICS";
    let actual = key_profiles::interrupted_key(text, &p, false)?;
    verified_key_text(
        "cipher.interrupted_key.encode",
        text,
        serde_json::to_value(&p).expect("typed static parameters"),
        actual,
        "HYIFQZPUKVQRBSEIJEQKZTVOBPOSZVSGSIICUIPY",
    )
}

pub(crate) fn interrupted_key_decode() -> Result<Fixture, OperationError> {
    let p = key_profiles::InterruptedKey {
        key: "ORANGE".into(),
        resets: vec![4, 10, 12, 15, 19, 22, 23, 24, 29, 30, 32, 35],
    };
    let text = "HYIFQZPUKVQRBSEIJEQKZTVOBPOSZVSGSIICUIPY";
    let actual = key_profiles::interrupted_key(text, &p, true)?;
    verified_key_text(
        "cipher.interrupted_key.decode",
        text,
        serde_json::to_value(&p).expect("typed static parameters"),
        actual,
        "THISCIPHERCANBEUSEDWITHANYOFTHEPERIODICS",
    )
}

pub(crate) fn progressive_key_encode() -> Result<Fixture, OperationError> {
    let p = key_profiles::ProgressiveKey {
        key: "GRAPEFRUIT".into(),
        increment: 1,
    };
    let text = "THISCIPHERCANBEUSEDWITHANYOFTHEPERIODICS";
    let actual = key_profiles::progressive_key(text, &p, false)?;
    verified_key_text(
        "cipher.progressive_key.encode",
        text,
        serde_json::to_value(&p).expect("typed static parameters"),
        actual,
        "ZYIHGNGBMKJSORJAKZMQQMJRTFHBDCNJHJPWXFNO",
    )
}

pub(crate) fn progressive_key_decode() -> Result<Fixture, OperationError> {
    let p = key_profiles::ProgressiveKey {
        key: "GRAPEFRUIT".into(),
        increment: 1,
    };
    let text = "ZYIHGNGBMKJSORJAKZMQQMJRTFHBDCNJHJPWXFNO";
    let actual = key_profiles::progressive_key(text, &p, true)?;
    verified_key_text(
        "cipher.progressive_key.decode",
        text,
        serde_json::to_value(&p).expect("typed static parameters"),
        actual,
        "THISCIPHERCANBEUSEDWITHANYOFTHEPERIODICS",
    )
}

pub(crate) fn vigenere_multiplicative_encode() -> Result<Fixture, OperationError> {
    let p = key_profiles::Multiplicative {
        key: "MOBILE".into(),
        multiplier: 7,
    };
    let text = "IAMSTUDYING";
    let actual = key_profiles::multiplicative(text, &p, false)?;
    verified_key_text(
        "cipher.vigenere_multiplicative.encode",
        text,
        serde_json::to_value(&p).expect("typed static parameters"),
        actual,
        "KUNACMBGLRP",
    )
}

pub(crate) fn vigenere_multiplicative_decode() -> Result<Fixture, OperationError> {
    let p = key_profiles::Multiplicative {
        key: "MOBILE".into(),
        multiplier: 7,
    };
    let text = "KUNACMBGLRP";
    let actual = key_profiles::multiplicative(text, &p, true)?;
    verified_key_text(
        "cipher.vigenere_multiplicative.decode",
        text,
        serde_json::to_value(&p).expect("typed static parameters"),
        actual,
        "IAMSTUDYING",
    )
}
pub(crate) fn gronsfeld_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"key":"9321492"}"##).expect("static parameters");
    let p: schedules::Gronsfeld =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "THISONEUSESTENOFTHETWENTYSIXVIGENEREALPHABETS";
    let actual = json!({"text":schedules::gronsfeld(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"CKKTSWGDVGTXNPXIVIICYNQVZWRZELIFRNTNDNQLJDNWU"}"##)
            .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.gronsfeld.encode: {actual} != {expected}"),
        )
        .operation("cipher.gronsfeld.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn gronsfeld_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"key":"9321492"}"##).expect("static parameters");
    let p: schedules::Gronsfeld =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "CKKTSWGDVGTXNPXIVIICYNQVZWRZELIFRNTNDNQLJDNWU";
    let actual = json!({"text":schedules::gronsfeld(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"THISONEUSESTENOFTHETWENTYSIXVIGENEREALPHABETS"}"##)
            .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.gronsfeld.decode: {actual} != {expected}"),
        )
        .operation("cipher.gronsfeld.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn trithemius_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"start":0,"step":1}"##).expect("static parameters");
    let p: schedules::Trithemius =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABC";
    let actual = json!({"text":schedules::trithemius(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text":"ACE"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.trithemius.encode: {actual} != {expected}"),
        )
        .operation("cipher.trithemius.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn trithemius_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"start":0,"step":1}"##).expect("static parameters");
    let p: schedules::Trithemius =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ACE";
    let actual = json!({"text":schedules::trithemius(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text":"ABC"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.trithemius.decode: {actual} != {expected}"),
        )
        .operation("cipher.trithemius.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn autokey_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"seed":"PRIMER","feedback":"plaintext"}"##)
        .expect("static parameters");
    let p: schedules::Autokey = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "THEAUTOKEYCANBEUSEDWITHVIGENEREVARIANTBEAUFORTORPORTA";
    let actual = json!({"text":schedules::autokey(text,&p,false)?});
    let expected: Value = serde_json::from_str(
        r##"{"text":"IYMMYKHRIYWTBLISUEQXMNZZLCMGLMMBEEMRROBVIUSHSXOLUCIMO"}"##,
    )
    .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.autokey.encode: {actual} != {expected}"),
        )
        .operation("cipher.autokey.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn autokey_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"seed":"PRIMER","feedback":"plaintext"}"##)
        .expect("static parameters");
    let p: schedules::Autokey = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "IYMMYKHRIYWTBLISUEQXMNZZLCMGLMMBEEMRROBVIUSHSXOLUCIMO";
    let actual = json!({"text":schedules::autokey(text,&p,true)?});
    let expected: Value = serde_json::from_str(
        r##"{"text":"THEAUTOKEYCANBEUSEDWITHVIGENEREVARIANTBEAUFORTORPORTA"}"##,
    )
    .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.autokey.decode: {actual} != {expected}"),
        )
        .operation("cipher.autokey.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn running_key_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"key":"BCD"}"##).expect("static parameters");
    let p: schedules::RunningKey =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABC";
    let actual = json!({"text":schedules::running_key(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text":"BDF"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.running_key.encode: {actual} != {expected}"),
        )
        .operation("cipher.running_key.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn running_key_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"key":"BCD"}"##).expect("static parameters");
    let p: schedules::RunningKey =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "BDF";
    let actual = json!({"text":schedules::running_key(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text":"ABC"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.running_key.decode: {actual} != {expected}"),
        )
        .operation("cipher.running_key.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn porta() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"key":"PORTA"}"##).expect("static parameters");
    let p: schedules::Key = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ENCIPHERMENTISRECIPROCAL";
    let actual = json!({"text":schedules::porta(text,&p)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"YGXRCOYJVRGMQJEYWQGEHWVU"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.porta: {actual} != {expected}"),
        )
        .operation("cipher.porta"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn gromark_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"alphabet":"AJRXEBKSYGFPVIDOUMHQWNCLTZ","primer":"23452"}"##)
            .expect("static parameters");
    let p: stateful::Gromark = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "THEREAREUPTOTENSUBSTITUTESPERLETTER";
    let actual = json!({"text":stateful::gromark(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"NFYCKBTIJCNWZYCACJNAYNLQPWWSTWPJQFL"}"##)
            .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.gromark.encode: {actual} != {expected}"),
        )
        .operation("cipher.gromark.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn gromark_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"alphabet":"AJRXEBKSYGFPVIDOUMHQWNCLTZ","primer":"23452"}"##)
            .expect("static parameters");
    let p: stateful::Gromark = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "NFYCKBTIJCNWZYCACJNAYNLQPWWSTWPJQFL";
    let actual = json!({"text":stateful::gromark(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"THEREAREUPTOTENSUBSTITUTESPERLETTER"}"##)
            .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.gromark.decode: {actual} != {expected}"),
        )
        .operation("cipher.gromark.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn periodic_gromark_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"keyword":"ENIGMA","period":6}"##).expect("static parameters");
    let p: stateful::PeriodicGromark =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "WINTRYSHOWERSWILLCONTINUEFORTHENEXTFEWDAYSACCORDINGTOTHEFORECAST";
    let actual = json!({"text":stateful::periodic_gromark(text,&p,false)?});
    let expected: Value = serde_json::from_str(
        r##"{"text":"RHNAAXNRUZBNIUARXCRTPATBRLIGDSVCIRCVOYPVRAAZZMUSREQYEVMMURGWTLUD"}"##,
    )
    .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "known answer mismatch for cipher.periodic_gromark.encode: {actual} != {expected}"
            ),
        )
        .operation("cipher.periodic_gromark.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn periodic_gromark_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"keyword":"ENIGMA","period":6}"##).expect("static parameters");
    let p: stateful::PeriodicGromark =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "RHNAAXNRUZBNIUARXCRTPATBRLIGDSVCIRCVOYPVRAAZZMUSREQYEVMMURGWTLUD";
    let actual = json!({"text":stateful::periodic_gromark(text,&p,true)?});
    let expected: Value = serde_json::from_str(
        r##"{"text":"WINTRYSHOWERSWILLCONTINUEFORTHENEXTFEWDAYSACCORDINGTOTHEFORECAST"}"##,
    )
    .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "known answer mismatch for cipher.periodic_gromark.decode: {actual} != {expected}"
            ),
        )
        .operation("cipher.periodic_gromark.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn solitaire_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"deck":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54],"max_steps":1048576}"##).expect("static parameters");
    let p: stateful::Solitaire = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "AAAAAAAAAA";
    let actual = json!({"text":stateful::solitaire(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"EXKYIZSGEH"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.solitaire.encode: {actual} != {expected}"),
        )
        .operation("cipher.solitaire.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn solitaire_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"deck":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54],"max_steps":1048576}"##).expect("static parameters");
    let p: stateful::Solitaire = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "EXKYIZSGEH";
    let actual = json!({"text":stateful::solitaire(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"AAAAAAAAAA"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.solitaire.decode: {actual} != {expected}"),
        )
        .operation("cipher.solitaire.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn slidefair_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"key":"B"}"##).expect("static parameters");
    let p: schedules::Key = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "CA";
    let actual = json!({"text":tableaux::slidefair(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text":"ZD"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.slidefair.encode: {actual} != {expected}"),
        )
        .operation("cipher.slidefair.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn slidefair_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"key":"B"}"##).expect("static parameters");
    let p: schedules::Key = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ZD";
    let actual = json!({"text":tableaux::slidefair(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text":"CA"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.slidefair.decode: {actual} != {expected}"),
        )
        .operation("cipher.slidefair.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn portax() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"key":"EASY"}"##).expect("static parameters");
    let p: schedules::Key = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "THEEARLYBIRDGETSTHEWORMX";
    let actual = json!({"text":tableaux::portax(text,&p)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"NIJAMPBGQCWKHQJEUIKYMPAT"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.portax: {actual} != {expected}"),
        )
        .operation("cipher.portax"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn ragbaby_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"alphabet":"GROSBEAKCDFHILMNPQTUVWYZ","merge_aliases":true}"##)
            .expect("static parameters");
    let p: stateful::Ragbaby = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "WORD DIVISIONS ARE KEPT";
    let actual = json!({"text":stateful::ragbaby(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"YBBL HNGQDUFGL DEF HFYR"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.ragbaby.encode: {actual} != {expected}"),
        )
        .operation("cipher.ragbaby.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn ragbaby_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"alphabet":"GROSBEAKCDFHILMNPQTUVWYZ","merge_aliases":true}"##)
            .expect("static parameters");
    let p: stateful::Ragbaby = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "YBBL HNGQDUFGL DEF HFYR";
    let actual = json!({"text":stateful::ragbaby(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"WORD DIVISIONS ARE KEPT"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.ragbaby.decode: {actual} != {expected}"),
        )
        .operation("cipher.ragbaby.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn condi_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"alphabet":"VWXYZSTRANGEBCDFHIJKLMOPQU","shift":25}"##)
            .expect("static parameters");
    let p: stateful::Condi = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text =
        "OURSISAVERYGREENPASTIMETHEWIDEVARIETYOFCIPHERSWEUSECANALLBESOLVEDWITHPENCILANDPAPER";
    let actual = json!({"text":stateful::condi(text,&p,false)?});
    let expected:Value=serde_json::from_str(r##"{"text":"MORCPPDNBKEDJKPMRTDBQCRJPXCKTVBNHUYJGVBYSFDXKCRCESIUOJJYFRQIXIMBVHKQPDNMPSBYJQBTTNK"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.condi.encode: {actual} != {expected}"),
        )
        .operation("cipher.condi.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn condi_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"alphabet":"VWXYZSTRANGEBCDFHIJKLMOPQU","shift":25}"##)
            .expect("static parameters");
    let p: stateful::Condi = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text =
        "MORCPPDNBKEDJKPMRTDBQCRJPXCKTVBNHUYJGVBYSFDXKCRCESIUOJJYFRQIXIMBVHKQPDNMPSBYJQBTTNK";
    let actual = json!({"text":stateful::condi(text,&p,true)?});
    let expected:Value=serde_json::from_str(r##"{"text":"OURSISAVERYGREENPASTIMETHEWIDEVARIETYOFCIPHERSWEUSECANALLBESOLVEDWITHPENCILANDPAPER"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.condi.decode: {actual} != {expected}"),
        )
        .operation("cipher.condi.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn nicodemus_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"key":"CAT"}"##).expect("static parameters");
    let p: tableaux::Nicodemus = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "THEEARLYBIRDGETSTHEWORM";
    let actual = json!({"text":tableaux::nicodemus(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"HAYREVGNKIXKUWMTWMUGTAH"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.nicodemus.encode: {actual} != {expected}"),
        )
        .operation("cipher.nicodemus.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn nicodemus_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"key":"CAT"}"##).expect("static parameters");
    let p: tableaux::Nicodemus = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "HAYREVGNKIXKUWMTWMUGTAH";
    let actual = json!({"text":tableaux::nicodemus(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"THEEARLYBIRDGETSTHEWORM"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.nicodemus.decode: {actual} != {expected}"),
        )
        .operation("cipher.nicodemus.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn quagmire_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"profile":"iv","plain_alphabet":"SENORYABCDFGHIJKLMPQTUVWXZ","cipher_alphabet":"PERCTIONABDFGHJKLMQSUVWXYZ","indicator":"S","key":"EXTRA"}"##).expect("static parameters");
    let p: tableaux::Quagmire = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "THISONEEMPLOYSTHREEKEYWORDS";
    let actual = json!({"text":tableaux::quagmire(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"VBMRFCYISPMPBRRHEICXRREIGDX"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.quagmire.encode: {actual} != {expected}"),
        )
        .operation("cipher.quagmire.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn quagmire_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"profile":"iv","plain_alphabet":"SENORYABCDFGHIJKLMPQTUVWXZ","cipher_alphabet":"PERCTIONABDFGHJKLMQSUVWXYZ","indicator":"S","key":"EXTRA"}"##).expect("static parameters");
    let p: tableaux::Quagmire = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "VBMRFCYISPMPBRRHEICXRREIGDX";
    let actual = json!({"text":tableaux::quagmire(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"THISONEEMPLOYSTHREEKEYWORDS"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.quagmire.decode: {actual} != {expected}"),
        )
        .operation("cipher.quagmire.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn phillips_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"square":"DIAGOCBSLNEFHKMUTRQPVWXYZ","profile":"rows"}"##)
            .expect("static parameters");
    let p: tableaux::Phillips = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "SQUARESONEANDFIVEAREACTUALLYTHESAMEASARESQUARESTWOANDEIGHTTHEOVERALLPERIODISFORTY";
    let actual = json!({"text":tableaux::phillips(text,&p,false)?});
    let expected:Value=serde_json::from_str(r##"{"text":"KZWLYTGEDTQETARBTYGTLFXWLPPOXLTYKUTKGKYTKZWLYTGXSEQETIRZQAAQTCITYKPPVBLHEFHGREYXO"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.phillips.encode: {actual} != {expected}"),
        )
        .operation("cipher.phillips.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn phillips_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"square":"DIAGOCBSLNEFHKMUTRQPVWXYZ","profile":"rows"}"##)
            .expect("static parameters");
    let p: tableaux::Phillips = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "KZWLYTGEDTQETARBTYGTLFXWLPPOXLTYKUTKGKYTKZWLYTGXSEQETIRZQAAQTCITYKPPVBLHEFHGREYXO";
    let actual = json!({"text":tableaux::phillips(text,&p,true)?});
    let expected:Value=serde_json::from_str(r##"{"text":"SQUARESONEANDFIVEAREACTUALLYTHESAMEASARESQUARESTWOANDEIGHTTHEOVERALLPERIODISFORTY"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.phillips.decode: {actual} != {expected}"),
        )
        .operation("cipher.phillips.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn chaocipher_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"left":"HXUCZVAMDSLKPEFJRIGTWOBNYQ","right":"PTLNBQDEOYSFAVZKGJRIHWXUMC"}"##,
    )
    .expect("static parameters");
    let p: stateful::Chaocipher =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "WELLDONEISBETTERTHANWELLSAID";
    let actual = json!({"text":stateful::chaocipher(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text":"OAHQHCNYNXTSZJRRHJBYHQKSOUJY"}"##)
        .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.chaocipher.encode: {actual} != {expected}"),
        )
        .operation("cipher.chaocipher.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn chaocipher_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"left":"HXUCZVAMDSLKPEFJRIGTWOBNYQ","right":"PTLNBQDEOYSFAVZKGJRIHWXUMC"}"##,
    )
    .expect("static parameters");
    let p: stateful::Chaocipher =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "OAHQHCNYNXTSZJRRHJBYHQKSOUJY";
    let actual = json!({"text":stateful::chaocipher(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text":"WELLDONEISBETTERTHANWELLSAID"}"##)
        .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.chaocipher.decode: {actual} != {expected}"),
        )
        .operation("cipher.chaocipher.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn playfair_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"square":"LOGARITHMBCDEFKNPQSUVWXYZ"}"##)
        .expect("static parameters");
    let p: geometry::Playfair = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "COMEQUICKLYWENEXEDHELP";
    let actual = serde_json::to_value(geometry::playfair(text, &p, false)?).expect("typed result");
    let expected: Value = serde_json::from_str(
        r##"{"text":"DLHFSNCNCRZXCQQGFEEQON","prepared":"COMEQUICKLYWENEXEDHELP","fillers":[]}"##,
    )
    .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.playfair.encode: {actual} != {expected}"),
        )
        .operation("cipher.playfair.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn playfair_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"square":"LOGARITHMBCDEFKNPQSUVWXYZ"}"##)
        .expect("static parameters");
    let p: geometry::Playfair = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "DLHFSNCNCRZXCQQGFEEQON";
    let actual = serde_json::to_value(geometry::playfair(text, &p, true)?).expect("typed result");
    let expected: Value = serde_json::from_str(
        r##"{"text":"COMEQUICKLYWENEXEDHELP","prepared":"COMEQUICKLYWENEXEDHELP","fillers":[]}"##,
    )
    .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.playfair.decode: {actual} != {expected}"),
        )
        .operation("cipher.playfair.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn seriated_playfair_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"square":"LOGARITHMBCDEFKNPQSUVWXYZ","period":6}"##)
            .expect("static parameters");
    let p: geometry::SeriatedPlayfair =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "COMEQUICKLYWENEEDHELPIMMEDIATELYTOM";
    let actual =
        serde_json::to_value(geometry::seriated_playfair(text, &p, false)?).expect("typed result");
    let expected:Value=serde_json::from_str(r##"{"text":"NLBCSPCDFGXZQQCDCMGCGQTBHCFTRHFGWHGB","prepared":"COMEQUICKLYWENEEDHXELPIMMEDIATELYTOM","fillers":[18]}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "known answer mismatch for cipher.seriated_playfair.encode: {actual} != {expected}"
            ),
        )
        .operation("cipher.seriated_playfair.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn seriated_playfair_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"square":"LOGARITHMBCDEFKNPQSUVWXYZ","period":6}"##)
            .expect("static parameters");
    let p: geometry::SeriatedPlayfair =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "NLBCSPCDFGXZQQCDCMGCGQTBHCFTRHFGWHGB";
    let actual =
        serde_json::to_value(geometry::seriated_playfair(text, &p, true)?).expect("typed result");
    let expected:Value=serde_json::from_str(r##"{"text":"COMEQUICKLYWENEEDHXELPIMMEDIATELYTOM","prepared":"COMEQUICKLYWENEEDHXELPIMMEDIATELYTOM","fillers":[]}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "known answer mismatch for cipher.seriated_playfair.decode: {actual} != {expected}"
            ),
        )
        .operation("cipher.seriated_playfair.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn two_square_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"first":"DIALOGUEBCFHKMNPQRSTVWXYZ","second":"BIOGRAPHYCDEFKLMNQSTUVWXZ"}"##,
    )
    .expect("static parameters");
    let p: geometry::Squares = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ANOTHERDIGRAPHICSETUPX";
    let actual = json!({"text":geometry::two_square(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"IRRTEHMKGIMEQGRUNMMZSV"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.two_square.encode: {actual} != {expected}"),
        )
        .operation("cipher.two_square.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn two_square_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"first":"DIALOGUEBCFHKMNPQRSTVWXYZ","second":"BIOGRAPHYCDEFKLMNQSTUVWXZ"}"##,
    )
    .expect("static parameters");
    let p: geometry::Squares = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "IRRTEHMKGIMEQGRUNMMZSV";
    let actual = json!({"text":geometry::two_square(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"ANOTHERDIGRAPHICSETUPX"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.two_square.decode: {actual} != {expected}"),
        )
        .operation("cipher.two_square.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn four_square_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"first":"GRDLUEYFNVOAHPWMBIQXTCKSZ","second":"LICNVOTDPWGHEQXAMFSYRBKUZ"}"##,
    )
    .expect("static parameters");
    let p: geometry::Squares = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "COMEQUICKLYWENEEDHELPX";
    let actual = json!({"text":geometry::four_square(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"LEWIXAFNEXCUDXUVDPGXHZ"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.four_square.encode: {actual} != {expected}"),
        )
        .operation("cipher.four_square.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn four_square_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"first":"GRDLUEYFNVOAHPWMBIQXTCKSZ","second":"LICNVOTDPWGHEQXAMFSYRBKUZ"}"##,
    )
    .expect("static parameters");
    let p: geometry::Squares = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "LEWIXAFNEXCUDXUVDPGXHZ";
    let actual = json!({"text":geometry::four_square(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"COMEQUICKLYWENEEDHELPX"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.four_square.decode: {actual} != {expected}"),
        )
        .operation("cipher.four_square.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn hill_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","dimension":2,"matrix":[3,3,2,5],"padding":null}"##).expect("static parameters");
    let p: hill::Hill = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "HELP";
    let actual = serde_json::to_value(hill::hill(text, &p, false)?).expect("typed result");
    let expected: Value =
        serde_json::from_str(r##"{"text":"HIAT","padding_count":0}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.hill.encode: {actual} != {expected}"),
        )
        .operation("cipher.hill.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn hill_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","dimension":2,"matrix":[3,3,2,5],"padding":null}"##).expect("static parameters");
    let p: hill::Hill = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "HIAT";
    let actual = serde_json::to_value(hill::hill(text, &p, true)?).expect("typed result");
    let expected: Value =
        serde_json::from_str(r##"{"text":"HELP","padding_count":0}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.hill.decode: {actual} != {expected}"),
        )
        .operation("cipher.hill.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn digrafid_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"horizontal":"KEYWORDABCFGHIJLMNPQSTUVXZ#","vertical":"VDPEFQRGSTHUIJWCKXAMYLNZBO#","extra":"#","period":3,"padding":null}"##).expect("static parameters");
    let p: geometry::Digrafid = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "THISISTHEFORESTPRI";
    let actual = serde_json::to_value(geometry::digrafid(text, &p, false)?).expect("typed result");
    let expected: Value = serde_json::from_str(
        r##"{"text":"HJMXWSWJADWGFCSPYI","prepared":"THISISTHEFORESTPRI","padding_count":0}"##,
    )
    .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.digrafid.encode: {actual} != {expected}"),
        )
        .operation("cipher.digrafid.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn digrafid_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"horizontal":"KEYWORDABCFGHIJLMNPQSTUVXZ#","vertical":"VDPEFQRGSTHUIJWCKXAMYLNZBO#","extra":"#","period":3,"padding":null}"##).expect("static parameters");
    let p: geometry::Digrafid = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "HJMXWSWJADWGFCSPYI";
    let actual = serde_json::to_value(geometry::digrafid(text, &p, true)?).expect("typed result");
    let expected: Value = serde_json::from_str(
        r##"{"text":"THISISTHEFORESTPRI","prepared":"THISISTHEFORESTPRI","padding_count":0}"##,
    )
    .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.digrafid.decode: {actual} != {expected}"),
        )
        .operation("cipher.digrafid.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn tri_square_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"first":"NSFMUOAGPWVBHQXECIRYLDKTZ","second":"READINGBCFHKLMOPQSTUVWXYZ","middle":"PASTINOQRMLYZUEKXWVBHGFDC","rows":[3,2,4,3,2,2,3,4,1,2],"columns":[2,0,4,4,3,1,3,4,2,4]}"##).expect("static parameters");
    let p: geometry::TriSquare = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "THREEKEYSQUARESUSEDX";
    let actual = json!({"text":geometry::tri_square(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text":"RHLQXRLXOEVZBATXSERXDDIUAAABFZ"}"##)
        .expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.tri_square.encode: {actual} != {expected}"),
        )
        .operation("cipher.tri_square.encode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn tri_square_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"first":"NSFMUOAGPWVBHQXECIRYLDKTZ","second":"READINGBCFHKLMOPQSTUVWXYZ","middle":"PASTINOQRMLYZUEKXWVBHGFDC"}"##).expect("static parameters");
    let p: geometry::TriSquare = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "RHLQXRLXOEVZBATXSERXDDIUAAABFZ";
    let actual = json!({"text":geometry::tri_square(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text":"THREEKEYSQUARESUSEDX"}"##).expect("static answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.tri_square.decode: {actual} != {expected}"),
        )
        .operation("cipher.tri_square.decode"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters,
        outputs: actual,
    })
}

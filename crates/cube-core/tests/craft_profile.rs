use cube_core::operations::execution::{self, Budget};
use serde_json::json;

#[test]
fn untweaked_profiles_keep_executing_without_injected_tweak_defaults() {
    let parameters = json!({"profile":"des_fips46_v1","key_hex":"0113b970fd34f2ce"});
    let input = json!({"input":{"base64":"BZteCFHPFDo="}});
    let actual = execution::execute(
        "cipher.block.encrypt_blocks",
        1,
        &input,
        &parameters,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(actual, json!({"output":{"base64":"hqVg8Q7G2Fs="}}));
}

#[test]
fn craft_requires_its_exact_tweak_and_executes_the_designer_answer() {
    let parameters = json!({"profile":"craft_2020_v1","key_hex":"18f4eebdfced7841d9e0e38cfe6a9405","tweak_hex":"17f291a9a3aa0400"});
    let input = json!({"input":{"base64":"ASNFZ4mrze8="}});
    let actual = execution::execute(
        "cipher.block.encrypt_blocks",
        1,
        &input,
        &parameters,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(
        actual,
        json!({"output":{"base64":cube_core::encoding::base64_encode(&hex::decode("111df5062a8d88c9").unwrap())}})
    );
}

#[test]
fn tweak_validation_precedes_work_and_never_exposes_private_material() {
    use cube_core::ciphers::block_profiles::{self, Profile, Settings};
    use cube_core::errors::ErrorCode;
    for tweak in [None, Some(&b"SECRET"[..]), Some(&[0u8; 9][..])] {
        let error = Settings::new_with_tweak(Profile::Craft, &[0; 16], None, tweak).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidParameter);
        assert!(!format!("{error:?}").contains("SECRET"));
    }
    assert!(Settings::new_with_tweak(Profile::Idea, &[0; 16], None, Some(&[0; 8])).is_err());
    let good =
        json!({"profile":"craft_2020_v1","key_hex":"00".repeat(16),"tweak_hex":"00".repeat(8)});
    for value in [
        json!(null),
        json!(true),
        json!(7),
        json!("SECRET"),
        json!("00".repeat(9)),
    ] {
        let mut p = good.clone();
        p["tweak_hex"] = value;
        assert!(Settings::parse(&p.to_string()).is_err());
    }
    let duplicate = r#"{"profile":"craft_2020_v1","key_hex":"00000000000000000000000000000000","tweak_hex":"0000000000000000","tweak_hex":"0000000000000000"}"#;
    assert!(Settings::parse(duplicate).is_err());
    let mut padded = good.to_string();
    padded.push_str(&" ".repeat(block_profiles::MAX_PARAMETERS - padded.len()));
    let settings = Settings::parse(&padded).unwrap();
    assert_eq!(
        block_profiles::encrypt(&[], &settings).unwrap(),
        Vec::<u8>::new()
    );
    padded.push(' ');
    assert_eq!(
        Settings::parse(&padded).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    for operation in [block_profiles::ENCRYPT, block_profiles::DECRYPT] {
        let c = cube_core::operations::require(operation)
            .unwrap()
            .contract()
            .unwrap();
        c.validate_parameters(&good).unwrap();
        let mut missing = good.clone();
        missing.as_object_mut().unwrap().remove("tweak_hex");
        assert!(c.validate_parameters(&missing).is_err());
        assert!(
            c.policy
                .sensitive_parameters
                .iter()
                .any(|name| name == "tweak_hex")
        );
    }
}

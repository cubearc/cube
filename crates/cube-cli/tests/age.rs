use std::process::Command;
#[test]
fn age_releases_only_complete_authenticated_plaintext() {
    let root = std::env::temp_dir().join(format!("cube-age-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let p = root.join("identity.json");
    let f = root.join("input.age");
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../cube-core/src/ciphers/age/vectors.json")).unwrap();
    for v in cases {
        std::fs::write(&p, v["parameters"].to_string()).unwrap();
        let data =
            cube_core::encoding::base64_decode(v["ciphertext_base64"].as_str().unwrap()).unwrap();
        std::fs::write(&f, &data).unwrap();
        let o = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["cipher", "age", "decrypt", "--parameters-file"])
            .arg(&p)
            .arg("--file")
            .arg(&f)
            .output()
            .unwrap();
        if v["expect"] == "success" {
            assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
            assert_eq!(
                o.stdout,
                cube_core::encoding::base64_decode(v["plaintext_base64"].as_str().unwrap())
                    .unwrap()
            );
        } else {
            assert!(!o.status.success());
            assert!(o.stdout.is_empty(), "{}", v["case"]);
            assert!(
                !String::from_utf8_lossy(&o.stderr)
                    .contains(v["parameters"]["identity"].as_str().unwrap())
            );
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

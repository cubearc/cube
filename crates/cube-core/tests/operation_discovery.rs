use cube_core::{operations, pipeline};
use std::collections::BTreeMap;

#[test]
fn discovery_preserves_descriptors_and_matches_the_executable_allowlist() {
    let adapters: BTreeMap<_, _> = pipeline::supported_adapters()
        .into_iter()
        .map(|adapter| (adapter.operation, adapter.version))
        .collect();
    for descriptor in operations::catalog() {
        let discovery = descriptor.discovery();
        assert!(std::ptr::eq(discovery.descriptor, descriptor));
        assert_eq!(
            discovery.pipeline.map(|support| support.version),
            adapters.get(descriptor.id).copied()
        );
        let mut json = serde_json::to_value(discovery).unwrap();
        let support = json.as_object_mut().unwrap().remove("pipeline").unwrap();
        assert_eq!(json, serde_json::to_value(descriptor).unwrap());
        assert_eq!(support.is_null(), !adapters.contains_key(descriptor.id));
    }
    assert_eq!(pipeline::adapter_version("encoding.hex.encode"), Some(1));
    for id in [
        "file.info.inspect",
        "image.decode",
        "image.bitwise",
        "pipeline.execute",
    ] {
        assert!(
            operations::require(id)
                .unwrap()
                .discovery()
                .pipeline
                .is_none()
        );
        assert_eq!(pipeline::adapter_version(id), None);
        assert!(
            pipeline::Plan::validate(
                pipeline::Kind::Bytes,
                vec![pipeline::Step::new(id)],
                pipeline::Limits::default(),
            )
            .is_err()
        );
    }
    assert_eq!(pipeline::adapter_version("missing.operation"), None);
    assert!(operations::require("missing.operation").is_err());
}

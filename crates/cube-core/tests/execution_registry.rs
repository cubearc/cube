use cube_core::{
    image::{ColorModel, Image},
    operations::{
        self,
        execution::{self, Budget},
    },
};
use serde_json::json;
#[test]
fn registered_executors_use_real_named_contracts_and_reject_unregistered_operations() {
    let budget = Budget::default();
    let catalog = execution::catalog(budget).unwrap();
    assert!(catalog.iter().any(|e| e.operation == "image.pixels.export"));
    assert!(execution::describe("file.info.inspect", budget).is_err());
    let fixtures: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/operations/contracts/conformance.json")).unwrap();
    for entry in catalog {
        let c = operations::require(entry.operation)
            .unwrap()
            .contract()
            .unwrap();
        let f = fixtures
            .iter()
            .find(|f| f["operation"] == entry.operation)
            .unwrap();
        let actual = execution::execute(
            entry.operation,
            c.operation_version,
            &f["inputs"],
            &f["parameters"],
            budget,
        )
        .unwrap();
        assert_eq!(
            operations::contracts::canonical_value(actual),
            operations::contracts::canonical_value(f["outputs"].clone()),
            "{}",
            entry.operation
        );
    }
}
#[test]
fn spiral_and_an_unrelated_executor_need_only_core_registration() {
    let b = Budget::default();
    assert_eq!(
        execution::execute("text.reverse", 1, &json!({"input":"abc"}), &json!({}), b).unwrap(),
        json!({"output":"cba"})
    );
    let im = Image::from_bytes(3, 3, ColorModel::Gray, b"Gur!! lrx", &[], &[]).unwrap();
    let input = json!({"input":operations::contracts::values::Image::from(&im)});
    for (traversal, expected) in [
        ("spiral_cw", b"Gur xrl!!".as_slice()),
        ("spiral_ccw", b"G!lrx ru!".as_slice()),
    ] {
        let out = execution::execute(
            "image.pixels.export",
            1,
            &input,
            &json!({"traversal":traversal}),
            b,
        )
        .unwrap();
        assert_eq!(
            out["data"]["base64"],
            cube_core::encoding::base64_encode(expected)
        );
        assert_eq!(out["output"]["traversal"], traversal);
    }
}
#[test]
fn budgets_versions_and_types_fail_without_bypassing_core_contracts() {
    let b = Budget {
        input_bytes: 8,
        output_bytes: 4,
        ..Budget::default()
    };
    assert!(execution::execute("text.reverse", 2, &json!({"input":"abc"}), &json!({}), b).is_err());
    assert!(execution::execute("text.reverse", 1, &json!({"input":7}), &json!({}), b).is_err());
    assert!(
        execution::execute(
            "text.reverse",
            1,
            &json!({"input":"123456789"}),
            &json!({}),
            b
        )
        .is_err()
    );
    assert!(
        execution::execute("text.reverse", 1, &json!({"input":"12345"}), &json!({}), b).is_err()
    );
}

#[test]
fn registered_interpreter_uses_effective_cell_eof_and_host_budget_settings() {
    let b = Budget::default();
    for (source, params, expected) in [
        ("-.", json!({"cell_bits":16}), vec![255]),
        (",.", json!({"eof":"zero"}), vec![0]),
        ("+,.", json!({"eof":"unchanged"}), vec![1]),
    ] {
        let value = execution::execute(
            "esolang.brainfuck.execute",
            2,
            &json!({"source":source,"input":[]}),
            &params,
            b,
        )
        .unwrap();
        assert_eq!(
            value,
            json!({"output":{"base64":cube_core::encoding::base64_encode(&expected)}})
        );
    }
    let b = Budget {
        output_bytes: 4,
        ..b
    };
    let defaults = execution::defaults("esolang.brainfuck.execute", b).unwrap();
    assert_eq!(defaults["max_output_bytes"], 4);
    assert!(
        execution::validate_parameters(
            "esolang.brainfuck.execute",
            2,
            &json!({"max_output_bytes":5}),
            b
        )
        .is_err()
    );
    assert!(
        execution::validate_parameters("esolang.brainfuck.execute", 2, &json!({"cell_bits":64}), b)
            .is_err()
    );
}

#[test]
fn image_selection_and_output_budgets_apply_to_the_same_registered_export() {
    let im = Image::from_bytes(3, 3, ColorModel::Gray, b"Gur!! lrx", &[], &[]).unwrap();
    let inputs = json!({"input":operations::contracts::values::Image::from(&im)});
    let p = json!({"traversal":"spiral_cw","channels":["gray"],"region":[0,0,3,3],"reverse":true});
    let output =
        execution::execute("image.pixels.export", 1, &inputs, &p, Budget::default()).unwrap();
    assert_eq!(
        output["data"]["base64"],
        cube_core::encoding::base64_encode(b"!!lrx ruG")
    );
    assert!(
        execution::execute(
            "image.pixels.export",
            1,
            &inputs,
            &json!({"channels":["red"]}),
            Budget::default()
        )
        .is_err()
    );
    assert!(
        execution::execute(
            "image.pixels.export",
            1,
            &inputs,
            &json!({"region":[0,0,4,3]}),
            Budget::default()
        )
        .is_err()
    );
    assert!(
        execution::execute(
            "image.pixels.export",
            1,
            &inputs,
            &p,
            Budget {
                output_bytes: 4,
                ..Budget::default()
            }
        )
        .is_err()
    );
    assert!(
        execution::execute(
            "image.pixels.export",
            1,
            &inputs,
            &p,
            Budget {
                image_sample_bytes: 8,
                ..Budget::default()
            }
        )
        .is_err()
    );
}

#[test]
fn host_defaults_do_not_make_required_contract_parameters_optional() {
    assert!(
        execution::validate_parameters("image.pixels.read", 1, &json!({}), Budget::default())
            .is_err()
    );
    assert!(
        execution::validate_parameters(
            "text.reverse",
            1,
            &json!({"shell":true}),
            Budget::default()
        )
        .is_err()
    );
}

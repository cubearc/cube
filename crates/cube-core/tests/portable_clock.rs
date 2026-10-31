use cube_core::{
    investigation::{
        profile::{Budgets, Profile},
        session::State,
    },
    pipeline,
};
use std::time::Duration;

#[test]
fn supplied_monotonic_clock_enforces_the_real_profile_budget() {
    let profile = Profile {
        budgets: Budgets {
            max_elapsed_ms: 10,
            ..Budgets::default()
        },
        ..Profile::default()
    };
    let mut reads = 0;
    let result = profile
        .run_bytes_with_clock(b"ABC", None, || {
            reads += 1;
            Ok(Duration::from_millis(if reads == 1 { 0 } else { 10 }))
        })
        .unwrap();
    assert_eq!(result.operation_units, 0);
    assert!(!result.session.runs().is_empty());
    assert!(
        result
            .session
            .runs()
            .iter()
            .all(|run| run.state == State::Skipped)
    );
    assert_eq!(result.elapsed_ms, 10);
}
#[test]
fn backwards_clock_is_rejected_instead_of_weakening_time_limits() {
    let profile = Profile {
        identify: true,
        ..Profile::default()
    };
    let mut values = [0, 5, 4].into_iter();
    assert!(
        profile
            .run_bytes_with_clock(b"ABC", None, || Ok(Duration::from_millis(
                values.next().unwrap_or(4)
            )))
            .is_err()
    );
}
#[test]
fn unavailable_clock_propagates_a_typed_failure() {
    let error = Profile::default()
        .run_bytes_with_clock(b"ABC", None, || {
            Err(pipeline::Error::from_operation(
                cube_core::errors::OperationError::new(
                    cube_core::errors::ErrorCode::Io,
                    "clock unavailable",
                ),
            ))
        })
        .unwrap_err();
    assert_eq!(error.error.code, cube_core::errors::ErrorCode::Io);
}

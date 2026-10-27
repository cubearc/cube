use cube_core::{errors::ErrorCode, numbers::combinatorics as c};
use num_bigint::BigUint;
fn choose(n: u64, k: u64) -> BigUint {
    let mut v = BigUint::from(1u8);
    for i in 1..=k {
        v *= n - k + i;
        v /= i;
    }
    v
}
#[test]
fn exact_count_oracles_symmetry_pascal_and_full_width() {
    for n in 0..=128 {
        for k in 0..=n {
            let r = c::binomial(n, k, c::Limits::default()).unwrap();
            assert_eq!(r.value.to_string(), choose(n, k).to_string());
            assert_eq!(
                r.value,
                c::binomial(n, n - k, c::Limits::default()).unwrap().value
            );
            assert!(serde_json::to_vec(&r).unwrap().len() <= r.report_bytes_upper_bound);
            if k > 0 && k < n {
                assert_eq!(
                    r.value,
                    c::binomial(n - 1, k - 1, c::Limits::default())
                        .unwrap()
                        .value
                        + c::binomial(n - 1, k, c::Limits::default()).unwrap().value
                );
            }
        }
    }
    assert_eq!(
        c::permutations(5, 2, c::Limits::default()).unwrap().value,
        20
    );
    assert_eq!(c::binomial(5, 2, c::Limits::default()).unwrap().value, 10);
    assert_eq!(
        c::multinomial(&[2, 1, 1], c::Limits::default())
            .unwrap()
            .value,
        12
    );
    for n in 0..=128 {
        for k in 0..=n {
            let expected = (0..k).fold(BigUint::from(1u8), |v, i| v * (n - i));
            match expected.to_string().parse::<u128>() {
                Ok(v) => assert_eq!(
                    c::permutations(n, k, c::Limits::default()).unwrap().value,
                    v
                ),
                Err(_) => assert_eq!(
                    c::permutations(n, k, c::Limits::default())
                        .unwrap_err()
                        .code,
                    ErrorCode::ResourceLimit
                ),
            }
        }
    }
    for groups in [&[][..], &[0], &[0, 0, 0]] {
        assert_eq!(
            c::multinomial(groups, c::Limits::default()).unwrap().value,
            1
        );
    }
    assert!(c::permutations(128, 128, c::Limits::default()).is_err());
    let triangle = c::pascal(129, c::MAX_WORK, c::MAX_TRIANGLE_BYTES).unwrap();
    assert_eq!(triangle.rows.len(), 129);
    for (r, row) in triangle.rows.iter().enumerate() {
        for (k, &v) in row.iter().enumerate() {
            assert_eq!(
                v,
                c::binomial(r as u64, k as u64, c::Limits::default())
                    .unwrap()
                    .value
            );
        }
    }
    assert!(serde_json::to_vec(&triangle).unwrap().len() <= triangle.report_bytes_upper_bound);
    assert!(
        c::pascal(0, 0, c::MAX_TRIANGLE_BYTES)
            .unwrap()
            .rows
            .is_empty()
    );
}
#[test]
fn count_domains_and_exact_one_over_budgets() {
    for (n, k) in [(129, 0), (1, 2), (u64::MAX, 0)] {
        assert_eq!(
            c::binomial(n, k, c::Limits::default()).unwrap_err().code,
            ErrorCode::InvalidParameter
        );
        assert!(c::permutations(n, k, c::Limits::default()).is_err());
    }
    for groups in [vec![1; 129], vec![u64::MAX, 1], vec![128, 1]] {
        assert!(c::multinomial(&groups, c::Limits::default()).is_err());
    }
    for l in [
        c::Limits {
            max_work: c::MAX_WORK + 1,
            ..c::Limits::default()
        },
        c::Limits {
            max_report_bytes: c::MAX_REPORT_BYTES + 1,
            ..c::Limits::default()
        },
    ] {
        assert!(c::binomial(1, 1, l).is_err());
    }
    for r in [
        c::binomial(128, 64, c::Limits::default()).unwrap(),
        c::permutations(20, 10, c::Limits::default()).unwrap(),
        c::multinomial(&[2, 3, 4], c::Limits::default()).unwrap(),
    ] {
        let l = c::Limits {
            max_work: r.work,
            max_report_bytes: r.report_bytes_upper_bound,
        };
        let call = |l| match r.value {
            v if v == c::binomial(128, 64, c::Limits::default()).unwrap().value => {
                c::binomial(128, 64, l)
            }
            v if v == c::permutations(20, 10, c::Limits::default()).unwrap().value => {
                c::permutations(20, 10, l)
            }
            _ => c::multinomial(&[2, 3, 4], l),
        };
        assert_eq!(call(l).unwrap().value, r.value);
        assert!(
            call(c::Limits {
                max_work: l.max_work - 1,
                ..l
            })
            .is_err()
        );
        assert!(
            call(c::Limits {
                max_report_bytes: l.max_report_bytes - 1,
                ..l
            })
            .is_err()
        );
    }
    let r = c::pascal(129, c::MAX_WORK, c::MAX_TRIANGLE_BYTES).unwrap();
    assert!(c::pascal(129, r.work, r.report_bytes_upper_bound).is_ok());
    assert!(c::pascal(129, r.work - 1, r.report_bytes_upper_bound).is_err());
    assert!(c::pascal(129, r.work, r.report_bytes_upper_bound - 1).is_err());
    assert!(c::pascal(130, c::MAX_WORK, c::MAX_TRIANGLE_BYTES).is_err());
    assert!(c::pascal(1, c::MAX_WORK + 1, c::MAX_TRIANGLE_BYTES).is_err());
    assert!(c::pascal(1, c::MAX_WORK, usize::MAX).is_err());
}

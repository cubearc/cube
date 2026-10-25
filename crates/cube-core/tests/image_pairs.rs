use cube_core::image::{
    BitwiseOperation, ColorModel, Image,
    pairs::{self, PairOperation},
};
#[test]
fn pair_order_counts_and_outputs_equal_direct_calls() {
    let images: Vec<_> = (0..4)
        .map(|i| Image::from_bytes(2, 1, ColorModel::Gray, &[i, i + 1], &[], &[]).unwrap())
        .collect();
    for n in 0..=4 {
        let refs: Vec<_> = images[..n].iter().collect();
        let batch = pairs::execute(
            &refs,
            PairOperation::Bitwise(BitwiseOperation::Xor),
            28,
            1000,
        )
        .unwrap();
        assert_eq!(batch.plan.pair_count, n * n.saturating_sub(1) / 2);
        let mut seen = vec![];
        for entry in batch.results {
            seen.push((entry.left, entry.right));
            assert_eq!(
                Image::decode(entry.png.as_ref().unwrap())
                    .unwrap()
                    .samples(),
                images[entry.left]
                    .bitwise(&images[entry.right], BitwiseOperation::Xor)
                    .unwrap()
                    .samples()
            );
            assert_eq!(
                entry.name.unwrap(),
                format!("pair-{:02}-{:02}.png", entry.left, entry.right)
            );
        }
        assert!(seen.windows(2).all(|w| w[0] < w[1]));
    }
    let refs: Vec<_> = images.iter().collect();
    let batch = pairs::execute(&refs, PairOperation::Compare, 28, 1).unwrap();
    assert_eq!(batch.retained_differences, 1);
    assert_eq!(
        batch
            .results
            .iter()
            .map(|r| r.comparison.as_ref().unwrap().differing_pixels)
            .sum::<u64>(),
        12
    );
    assert!(!batch.results[1].comparison.as_ref().unwrap().complete);
    assert!(pairs::plan(&[&images[0], &images[0]], PairOperation::Compare, 28, 1).is_err());
    assert!(pairs::plan(&refs, PairOperation::Compare, 5, 1).is_err());
    assert!(pairs::pair_count(usize::MAX).is_err());
    if usize::BITS == 64 {
        let n = (u64::from(u32::MAX) + 2) as usize;
        assert_eq!(
            pairs::pair_count(n).unwrap() as u128,
            (n as u128) * (n as u128 - 1) / 2
        );
    }
}
#[test]
fn failures_are_per_pair_and_preflight_limits_include_failed_work() {
    let a = Image::from_bytes(2, 1, ColorModel::Gray, &[0, 1], &[], &[]).unwrap();
    let b = a.clone();
    let c = Image::from_bytes(1, 2, ColorModel::Gray, &[0, 1], &[], &[]).unwrap();
    let batch = pairs::execute(&[&a, &b, &c], PairOperation::Compare, 28, 1000).unwrap();
    assert_eq!((batch.successes, batch.failures), (1, 2));
    assert!(batch.results[0].comparison.as_ref().unwrap().equal);
    assert!(batch.results[1].error.is_some());
    for invalid in ["", "missing"] {
        assert!(PairOperation::parse(invalid).is_err());
    }
    assert!(pairs::plan(&[&a], PairOperation::Compare, 29, 1000).is_err());
    assert!(pairs::plan(&[&a], PairOperation::Compare, 28, 0).is_err());
    let images: Vec<_> = (0..4)
        .map(|_| {
            Image::from_samples(
                2048,
                2048,
                ColorModel::Gray,
                vec![0; 4 * 1024 * 1024],
                vec![],
                vec![],
            )
            .unwrap()
        })
        .collect();
    assert!(
        pairs::plan(
            &images.iter().collect::<Vec<_>>(),
            PairOperation::Compare,
            28,
            1
        )
        .is_err()
    );
}

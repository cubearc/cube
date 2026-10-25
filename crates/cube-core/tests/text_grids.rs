use cube_core::{
    image::{Image, Traversal},
    text::grid,
};
#[test]
fn factor_pairs_are_exact_ordered_and_bounded() {
    for (text, expected) in [
        ("A", vec![(1, 1)]),
        ("ABCDE", vec![(1, 5), (5, 1)]),
        ("ABCDEF", vec![(1, 6), (2, 3), (3, 2), (6, 1)]),
        ("ABCD", vec![(1, 4), (2, 2), (4, 1)]),
    ] {
        let report = grid::factors(text, false, 128).unwrap();
        assert_eq!(
            report
                .candidates
                .iter()
                .map(|d| (d.width, d.height))
                .collect::<Vec<_>>(),
            expected
        );
    }
    assert!(grid::factors("ABCD", false, 2).is_err());
    assert!(grid::factors("x", false, 0).is_err());
    assert!(grid::factors("", false, 128).is_err());
    assert!(grid::factors(&"a".repeat(65537), false, 128).is_err());
    assert!(grid::factors("\t \n", true, 128).is_err());
}
#[test]
fn traversal_filtering_and_quoted_symbols_preserve_scalars() {
    let g = grid::render("a e\u{301}🧊", 2, true, Traversal::Row).unwrap();
    assert_eq!(g.retained_scalars, 4);
    assert_eq!(g.removed_scalars, 1);
    assert_eq!(g.rows, vec![vec!["a", "e"], vec!["\u{301}", "🧊"]]);
    assert_eq!(
        grid::render("ABCDEF", 3, false, Traversal::Column)
            .unwrap()
            .rows,
        vec![vec!["A", "C", "E"], vec!["B", "D", "F"]]
    );
    let literal = grid::render("\n\t", 2, false, Traversal::Row).unwrap();
    assert_eq!(literal.text().unwrap(), "\"\\n\"\t\"\\t\"\n");
    for width in [0, 4, u32::MAX] {
        assert!(grid::render("ABCDEF", width, false, Traversal::Row).is_err());
    }
    for n in [2, 3, 4] {
        let text: String = (0..n).map(|i| char::from(b'0' + i)).collect();
        let g = grid::render(&text, n as u32, false, Traversal::Row).unwrap();
        let colors: Vec<_> = (0..n)
            .map(|i| ((char::from(b'0' + i)).to_string(), [i, i, i, 255]))
            .collect();
        let image = g.image(&colors).unwrap();
        assert_eq!(image.samples()[0], 0);
        assert_eq!(
            Image::decode(&image.encode_png().unwrap())
                .unwrap()
                .samples(),
            image.samples()
        );
    }
    let g = grid::render("01", 2, false, Traversal::Row).unwrap();
    for c in [
        vec![],
        vec![("0".into(), [0; 4])],
        vec![("0".into(), [0; 4]), ("0".into(), [1; 4])],
    ] {
        assert!(g.image(&c).is_err());
    }
    assert!(grid::parse_colors("[[\"0\",\"000000ff\"],[\"1\",\"ffffffff\"]]").is_ok());
    for bad in [
        "{}",
        "[[\"ab\",\"000000ff\"]]",
        "[[\"0\",\"xx\"]]",
        "[[\"0\",\"zzzzzzzz\"]]",
    ] {
        assert!(grid::parse_colors(bad).is_err());
    }
}

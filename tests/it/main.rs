use expect_test::{expect, Expect};
use hotsauce::Regex;

mod external;

fn check(pat: &str, hay: &str, expect: Expect) {
    let actual = Regex::new(pat)
        .unwrap()
        .matches(hay.bytes())
        .collect::<Vec<_>>();
    expect.assert_debug_eq(&actual);
}

#[test]
fn no_match() {
    let pat = "hello";
    let hay = "world";

    let expect = expect![[r#"
        []
    "#]];

    check(pat, hay, expect);
}

#[test]
fn no_match_empty() {
    let pat = "hello";
    let hay = "";

    let expect = expect![[r#"
        []
    "#]];

    check(pat, hay, expect);
}

#[test]
fn single_match() {
    let pat = "hey";
    let hay = " hey ";

    let expect = expect![[r#"
        [
            1..4,
        ]
    "#]];

    check(pat, hay, expect);
}

#[test]
fn multi_match() {
    let pat = "hey";
    let hay = "hey hey";

    let expect = expect![[r#"
        [
            0..3,
            4..7,
        ]
    "#]];

    check(pat, hay, expect);
}

#[test]
fn overlapping() {
    let pat = "aa";
    let hay = "aaa";

    let expect = expect![[r#"
        [
            0..2,
        ]
    "#]];

    check(pat, hay, expect);
}

#[test]
fn search_backwards_from_end() {
    let pat = "hey";
    let hay = "hey hey";

    let expect = expect![[r#"
        [
            0..3,
            4..7,
        ]
    "#]];

    let actual = Regex::new(pat)
        .unwrap()
        .rmatches(hay.bytes().rev())
        .collect::<Vec<_>>();

    expect.assert_debug_eq(&actual);
}

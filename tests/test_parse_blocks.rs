use mineclaw::tools::filesystem::parse_search_replace_blocks_from_diff;

#[test]
fn test_parse_search_replace_blocks() {
    let diff = r#"
------- SEARCH
Line A: foo
=======
Line A: FOO
+++++++ REPLACE

------- SEARCH
Line C: foo
=======
Line C: FOO
+++++++ REPLACE
"#;
    let blocks2 = parse_search_replace_blocks_from_diff(diff);
    assert_eq!(blocks2.len(), 2);
    assert_eq!(
        blocks2[0],
        ("Line A: foo".to_string(), "Line A: FOO".to_string())
    );
    assert_eq!(
        blocks2[1],
        ("Line C: foo".to_string(), "Line C: FOO".to_string())
    );
}

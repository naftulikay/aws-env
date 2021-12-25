use std::borrow::Cow;

const DATA: &'static str = r#"
[section]
key=value
"#;

#[test]
fn test_accepts_cow() {
    let d = DATA.to_string();
    let cow = Cow::from(d);
    let _ = super::parse(&cow);
}

#[test]
fn test_accepts_string() {
    let d = DATA.to_string();
    let _ = super::parse(&d);
}

#[test]
fn test_accepts_str() {
    let _ = super::parse(DATA);
}

use trybuild::TestCases;

#[test]
fn ui() {
    let t = TestCases::new();

    t.pass("tests/ui/00-initial-concept.rs");
    t.pass("tests/ui/01-tupled-struct.rs");
    t.pass("tests/ui/02-named-struct.rs");
}

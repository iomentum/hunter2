#![allow(dead_code)]

use hunter2::Hunter2;

#[derive(Hunter2)]
struct Credentials(#[hidden] String);

fn main() {
    let credentials = Credentials("ferris is cute".to_string());
    let debug_repr = format!("{:#?}", credentials);

    assert_eq!(
        debug_repr,
        "Credentials(\n    \
            ****************,\n\
        )"
    );
}

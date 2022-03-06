#![allow(dead_code)]

use hunter2::Hunter2;

#[derive(Hunter2)]
struct Credentials {
    username: String,
    #[hidden]
    password: String,
}

fn main() {
    let credentials = Credentials {
        username: "scrabsha".to_string(),
        password: "ferris is cute".to_string()
    };
    let debug_repr = format!("{:#?}", credentials);

    assert_eq!(
        debug_repr,
        "Credentials {\n    \
            username: \"scrabsha\",\n    \
            password: ****************,\n\
        }",
    );
}

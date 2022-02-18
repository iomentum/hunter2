#![allow(dead_code)]

use hunter2::Hunter2;

#[derive(Hunter2)]
struct AppSettings {
    #[hidden]
    db_password: &'static str,
    max_concurrent_users: u64,
}

fn main() {
    let credentials = AppSettings {
        db_password: "ferris is cute",
        max_concurrent_users: 254,
    };

    let debug_repr = format!("{:#?}", credentials);

    assert_eq!(
        debug_repr,
        "AppSettings {\n    \
            db_password: ****************,\n    \
            max_concurrent_users: 254,\n\
        }"
    );
}

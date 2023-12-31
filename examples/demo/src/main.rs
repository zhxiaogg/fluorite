mod demo {
    include!(concat!(env!("OUT_DIR"), "/demo/mod.rs"));
}
use demo::{Gender, User};
fn main() {
    let user = User {
        first_name: "f".to_string(),
        last_name: "l".to_string(),
        gender: Gender::Male,
        age: 10,
        active: true,
        info: None,
    };
    println!("user: {:?}", user);
}

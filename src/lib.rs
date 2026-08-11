use tokio::runtime::Runtime;
use crate::auth::{auth_xbox_live, request_auth_code};

mod auth;

#[test]
fn test_xbox_auth() {
    let live = Runtime::new().unwrap().block_on(auth_xbox_live());

    println!("{:#?}", live);

    assert!(live.is_ok())
}

#[test]
fn test_oauth2() {
    let oauth = request_auth_code();

    println!("{:#?}", oauth);

    assert!(oauth.is_ok())
}
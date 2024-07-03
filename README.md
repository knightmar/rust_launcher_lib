# Rust Launcher Lib

This library was created to allow the creation of launchers for the minecraft video game, using the rust programming language.

## Code organisation

The code of this project is organized in a simple way: 
- src/auth: code for authenticating game accounts
- src/launch: code to launch the game once the files have been downloaded
- src/update: code for updating game files
- lib.rs: main library file

## Librairies used
Here are the main librairies that i'm using on this project :
- [mc_auth](https://docs.rs/mc_auth/0.1.0/mc_auth/)
- [serde](https://docs.rs/serde/1.0.203/serde/)
- [reqwest](https://docs.rs/reqwest/latest/reqwest/)
- [futures](https://docs.rs/futures/latest/futures/)
- [tokio](https://docs.rs/tokio/latest/tokio/)

(The other libs I use can be found in cargo.toml file)

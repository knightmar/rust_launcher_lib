
# Rust Launcher Lib

This library was created to allow the creation of launchers for the minecraft video game, using the rust programming language.

## Code organisation

The code of this project is organized in a simple way: 
- src/auth: code for authenticating game accounts
- src/launch: code to launch the game once the files have been downloaded
- src/update: code for updating game files
- lib.rs: main library file

## How to use
First install this librairy into your project
### Install
Then, to launch the game we need an Updater object (I'll use the 1.21 version of the game) : 
```rust
let mut updater = Updater::new("1.21");
```
We need to specify the location of our launcher (In the AppData folder) : 
```rust
updater.set_relative_local_dir_path(".rustLauncherLib");
```
And then install the game files : 
```rust 
updater.install_files();
``` 
### Launch
For the moment, we have our files on the disk, let's launch the game : 
```rust 
let launcher = launch::GameLauncher::new(
    "1.21".to_string(),
    ".rustLauncherLib".to_string(),
    vec![], // the game arguments if you need to pass some
    vec![], // the jvm arguments
);
launcher.launch("access_token", "username").unwrap();
```
And here it is ! The game is launched.

For the people that don't want to understand what they are doing, here is the full code 😉 :

```rust
fn main() {
    let mut updater = Updater::new("1.21");
    updater.set_relative_local_dir_path(".rustLauncherLib");
    updater.install_files();

    let launcher = launch::GameLauncher::new(
        "1.21".to_string(),
        ".rustLauncherLib".to_string(),
        vec![],
        vec![],
    );
    launcher.launch("access_token", "username").unwrap();
}
```

### Authentification
// todo

## Librairies used
Here are the main librairies that i'm using on this project :
- [mc_auth](https://docs.rs/mc_auth/0.1.0/mc_auth/)
- [serde](https://docs.rs/serde/1.0.203/serde/)
- [reqwest](https://docs.rs/reqwest/latest/reqwest/)
- [futures](https://docs.rs/futures/latest/futures/)
- [tokio](https://docs.rs/tokio/latest/tokio/)

(The other libs I use can be found in cargo.toml file)

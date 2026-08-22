# Rust Launcher Lib

This library was created to allow the creation of launchers for the Minecraft video game, using the Rust programming language.

## Current state
The lib is in early development so many features aren't already available.
- [x] Vanilla install (all versions)
- [x] Game launch (only on vanilla)
- [x] Authentication (Microsoft and offline)
- [ ] Mod loader installation (Forge, Fabric, Quilt, Neoforge)
- [ ] Mod installation (CurseForge, Modrinth, personal server)
- [ ] Custom files
- [ ] Custom launching argument (JVM + mc)
- [ ] Proper documentation
- [ ] Callbacks for download / installation / verification

## Code organization

The code of this project is organized in a simple way: 
- src/auth: code for authenticating game accounts
- src/update: code for updating game files
- src/launch: code to launch the game once the files have been downloaded
- lib.rs: main library file

## How to use
First install this library into your project
### Install
Then, to launch the game we need an Updater object (I'll use the 1.21 version of the game) : 
```rust
let mut updater = Updater::new("26.2".to_string(), "/home/user/.your_dir".to_string());
```
And then install the game files, install_version() is async so you need to block on (may change in future): 
```rust 
tokio::runtime::Runtime::new()
.unwrap()
.block_on(updater.install_version());
``` 

### Launch
For the moment, we have our files on the disk, let's launch the game : 
```rust 
let launcher = Launcher::new(
updater.game_files_location.into(),
Some("your_player_uuid".to_string()), // None to play offline
Some("your_access_token".to_string()), // None to play offline
"username".to_string(),
"26.2".to_string(),
);
```
And here it is ! The game is launched.

For the people that don't want to understand what they are doing, here is the full code 😉 :

```rust
fn main() {
    let mut updater = Updater::new("26.2".to_string(), "/home/user/.your_dir".to_string());
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(updater.install_version());

    println!("{:#?}", result.err().ok_or("No error"));

    let launcher = Launcher::new(
        updater.game_files_location.into(),
        Some("your_player_uuid".to_string()), // None to play offline
        Some("your_access_token".to_string()), // None to play offline
        "username".to_string(),
        "26.2".to_string(),
    );

    println!("{:?}", launcher.launch());
}
```

### Authentification
No doc for now, but go check the code, it's not that hard

# License
MIT License

Copyright (c) 2026 knightmar

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

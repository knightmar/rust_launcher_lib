use std::path::Path;

pub struct LaunchBuilder {
    program:String,
    libs:String
}

impl LaunchBuilder {
    pub fn new<S: ToString>(program:S) -> LaunchBuilder {
        LaunchBuilder {
            program: program.to_string(),
            libs: "".to_string(),
        }
    }
    
    // get the string for the libs to use on the cmdline to launch the game 
    pub fn set_libs_to_launch<S: ToString>(&mut self, lib_path:S, client_path:S) {
        let mut lib_str = String::new();
        lib_str.push_str(&format!("{};", client_path.to_string()));

        for entry in std::fs::read_dir(Path::new(&lib_path.to_string())).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                lib_str.push_str(path.to_str().unwrap());
                lib_str.push(';');
            }
        }

        self.libs = lib_str;
    }

    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn libs(&self) -> &str {
        &self.libs
    }
}
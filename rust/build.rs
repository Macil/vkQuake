use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

fn compute_include_paths() -> Vec<String> {
    let mut include_paths: Vec<String> = vec![];

    if let Ok(include_path) = env::var("SDL2_INCLUDE_PATH") {
        include_paths.push(include_path);
    };

    if cfg!(target_os = "macos") {
        include_paths.push("/opt/homebrew/include".to_string());
    }

    if cfg!(target_os = "windows") {
        include_paths.push("../Windows/SDL2/include".to_string());
        include_paths.push("../Windows/misc/include".to_string());
    } else {
        let pkg_config_library = pkg_config::Config::new()
            .print_system_libs(false)
            .probe("sdl2")
            .unwrap();
        for path in pkg_config_library.include_paths {
            include_paths.push(path.to_string_lossy().to_string());
        }
    }

    include_paths
}

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

fn run_bindgen() {
    if cfg!(target_os = "windows") && env::var("LIBCLANG_PATH").is_err() {
        if let Ok(path) = env::var("PATH") {
            let mut extra_paths_to_check = Vec::new();

            if cfg!(target_arch = "x86_64") {
                extra_paths_to_check.push("C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\Llvm\\x64\\bin");
            } else if cfg!(target_arch = "x86") {
                extra_paths_to_check.push("C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\Llvm\\bin");
            } else if cfg!(target_arch = "aarch64") {
                extra_paths_to_check.push("C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\Llvm\\ARM64\\bin");
            }

            let libclang_path = path.split(';').chain(extra_paths_to_check).find(|path| {
                    PathBuf::from(path)
                        .join("libclang.dll")
                        .exists()
                }).expect("Failed to find libclang.dll in PATH. Make sure LLVM is installed, otherwise set LIBCLANG_PATH.");
            env::set_var("LIBCLANG_PATH", libclang_path);
        }
    }

    let ignored_macros = IgnoreMacros(
        vec![
            "FP_INFINITE".into(),
            "FP_NAN".into(),
            "FP_NORMAL".into(),
            "FP_SUBNORMAL".into(),
            "FP_ZERO".into(),
            "IPPORT_RESERVED".into(),
        ]
        .into_iter()
        .collect(),
    );

    let mut bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // Workaround these failing
        .layout_tests(false)
        // This is needed on msvc builds
        .blocklist_type(r".*IMAGE_TLS_DIRECTORY.*")
        // These are needed on mingw builds
        .blocklist_type(r"_JUMP_BUFFER")
        .blocklist_type(r"__mingw_ldbl_type_t")
        .blocklist_type(r"_complex")
        .blocklist_function(r"_cabs")
        .parse_callbacks(Box::new(ignored_macros));

    for include_path in compute_include_paths() {
        bindings = bindings.clang_arg(format!("-I{include_path}"));
    }

    let bindings = bindings
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    run_bindgen();
}

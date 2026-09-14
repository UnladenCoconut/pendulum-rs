use std::{env, path::PathBuf};

use walkdir::WalkDir;
use which::which;
fn main() {
    let slangc = which("slangc");
    let slangc = match slangc {
        Ok(s) => s,
        Err(e) => {
            panic!(
                "could not find the slang compiler slangc in the path: {}",
                e
            );
        }
    };

    println!("cargo::rerun-if-changed=src/shaders");
    WalkDir::new("./src/shaders")
        .into_iter()
        .filter_map(|x| match x {
            Ok(e) => {
                if e.file_name().to_str().unwrap().ends_with(".slang") {
                    Some(e)
                } else {
                    None
                }
            }
            Err(e) => {
                panic!("{}", e);
            }
        })
        .for_each(|x| {
            println!("cargo::rerun-if-changed={}", x.path().display());
            println!("cargo::warning=rebuilding shader {}", x.path().display());

            let mut xpb = PathBuf::from(x.path());
            xpb.set_extension("spirv");
            let spirv = PathBuf::from(env::var("OUT_DIR").unwrap()).join(xpb.file_name().unwrap());

            let args = &[
                x.path().to_str().unwrap(),
                "-target",
                "spirv",
                "-o",
                spirv.to_str().unwrap(),
            ];

            let mut sp = std::process::Command::new(slangc.clone());
            sp.args(args);

            let mut st = sp.spawn().unwrap();
            let ec = st.wait().unwrap();
            if !ec.success() {
                panic!("slangc failed with exit code {:?}", ec.code());
            }
        });
}

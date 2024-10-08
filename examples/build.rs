//! An Example of using ASN.1 compiler in `build.rs`
//!

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use log::{debug, error, log_enabled, info, Level};

use asn1_compiler::{
    generator::{Codec, Derive, Visibility},
    Asn1Compiler,
};

fn get_specs_files(
    specs_name: &str,
    specs_dir: std::path::PathBuf,
    prefix: &str,
) -> std::io::Result<Vec<PathBuf>> {
    let specs_files = specs_dir
        .read_dir()?
        // Following will get rid of non-ok values
        .flatten()
        // We now get the 'path' for the `DirEntry`
        .map(|file| file.path())
        // Now we filter all such paths such that
        .filter(|path| {
            path.file_name()
                // Filename is a valid one and if Valid is Option<&OsStr<>
                // is changed to Option<&str>
                .and_then(|f| f.to_str())
                // This is a `map` on the option - converts Option<&str>, Option<bool>,
                // Leaving `None` as it is
                .map(|s| s.starts_with(prefix))
                // If it's None, it's falsey, filter out
                .unwrap_or_default()
        })
        // Collect everything as Vec<Path>
        .collect::<Vec<_>>();

    eprintln!("specs_name:{} specs_files: {:#?}", specs_name, specs_files);

    Ok(specs_files)
}

fn main() -> std::io::Result<()> {
    let specs = vec!["ranap", "s1ap", "ngap", "e2ap", "supl","f1ap","rrc"];
    let modules = vec!["ranap.rs", "s1ap.rs", "ngap.rs", "e2ap.rs", "supl.rs","f1ap.rs","rrc.rs"];

    

    //let specs = vec!["f1ap"];
    //let modules = vec!["f1ap.rs"];
    let mut codecs_map = HashMap::new();
    codecs_map.insert("ranap.rs", vec![Codec::Aper]);
    codecs_map.insert("s1ap.rs", vec![Codec::Aper]);
    codecs_map.insert("ngap.rs", vec![Codec::Aper]);
    codecs_map.insert("e2ap.rs", vec![Codec::Aper]);
    codecs_map.insert("supl.rs", vec![Codec::Uper]);
    codecs_map.insert("f1ap.rs", vec![Codec::Aper]);
    codecs_map.insert("rrc.rs", vec![Codec::Aper]);

    println!("codecs_map: {:#?}", codecs_map);
    for (spec, module) in std::iter::zip(specs, modules) {
        let specs_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("specs")
            .join(spec);

        let specs_files = get_specs_files(spec, specs_dir, &spec.to_ascii_uppercase())?;

        let rs_module = PathBuf::from(env::var("OUT_DIR").unwrap()).join(module);
        let rs_module = rs_module.to_str().unwrap();
        let mut compiler = Asn1Compiler::new(
            rs_module,
            &Visibility::Public,
            codecs_map.get(module).unwrap().clone(),
            vec![
                Derive::Debug,
                Derive::EqPartialEq,
                Derive::Serialize,
                Derive::Deserialize,
            ],
        );
        debug!("compiling files");
        compiler.compile_files(&specs_files)?;
    }

    Ok(())
}

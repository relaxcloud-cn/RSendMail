fn main() {
    let manifest_dir =
        std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let config = slint_build::CompilerConfiguration::new().with_library_paths(
        std::collections::HashMap::from([(
            "material".to_string(),
            manifest_dir.join("material-1.0/material.slint"),
        )]),
    );

    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}

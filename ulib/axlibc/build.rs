use std::path::PathBuf;
fn main() {
    fn gen_c_to_rust_bindings(in_file: &str, out_file: &str) {
        let root_dir = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        println!("cargo:rerun-if-changed={in_file}");

        let allow_types = ["tm", "jmp_buf"];
        let path = root_dir.join("ulib/axlibc/include");
        let include_flag = "-I ";
        let include_str = format!("{}{}", include_flag, path.to_string_lossy());
        let include_str_ref = include_str.as_str();
        let mut builder = bindgen::Builder::default()
            .header(in_file)
            .clang_arg(include_str_ref)
            .derive_default(true)
            .size_t_is_usize(false)
            .use_core();
        for ty in allow_types {
            builder = builder.allowlist_type(ty);
        }

        builder
            .generate()
            .expect("Unable to generate c->rust bindings")
            .write_to_file(out_file)
            .expect("Couldn't write bindings!");
    }

    gen_c_to_rust_bindings("ctypes.h", "src/libctypes_gen.rs");
}

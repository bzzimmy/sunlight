use std::fs;

fn main() {
    let mut build = cc::Build::new();
    for module in ["Common", "Ast", "Bytecode", "Compiler"] {
        build.include(format!("luau/{module}/include"));
        for entry in fs::read_dir(format!("luau/{module}/src")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|extension| extension == "cpp") {
                build.file(path);
            }
        }
    }
    build
        .cpp(true)
        .std("c++17")
        .file("cpp/compile.cpp")
        .warnings(false)
        .compile("luau");
    println!("cargo::rerun-if-changed=cpp");
    println!("cargo::rerun-if-changed=luau");
}

use std::path::PathBuf;

fn build_langs(langs: &[(&str, Option<&str>, &[&str])]) {
    for (lang, subdir, files) in langs {
        let mut dir = PathBuf::new();
        let repo_name = format!("tree-sitter-{}", lang);
        dir.push("vendor");
        dir.push(&repo_name);
        if let Some(subdir) = subdir {
            dir.push(subdir);
        }
        dir.push("src");
        for file in *files {
            let loc = dir.join(file);
            println!("cargo:rerun-if-changed={}", loc.display());
            cc::Build::new()
                .include(&dir)
                .warnings(false)
                .file(loc)
                .compile(&format!("{}_{}", repo_name, file));
        }
    }
}

// https://doc.rust-lang.org/cargo/reference/build-scripts.html
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    build_langs(&[
        ("cpp", None, &["parser.c", "scanner.c"]),
        ("c", None, &["parser.c"]),
        // ("elixir", None, &["parser.c", "scanner.c"]),
        // ("elm", None, &["parser.c", "scanner.c"]),
        // ("haskell", None, &["parser.c", "scanner.c"]),
        ("javascript", None, &["parser.c", "scanner.c"]),
        ("markdown", None, &["parser.c", "scanner.c"]),
        // ("nix", None, &["parser.c", "scanner.c"]),
        // ("php", None, &["parser.c", "scanner.cc"]),
        // ("ruby", None, &["parser.c", "scanner.cc"]),
        ("python", None, &["parser.c", "scanner.c"]),
        ("rust", None, &["parser.c", "scanner.c"]),
        ("java", None, &["parser.c"]),
        ("go", None, &["parser.c"]),
        ("lua", None, &["parser.c", "scanner.c"]),
        // ("typescript", Some("typescript"), &["parser.c", "scanner.c"]),
    ]);
}

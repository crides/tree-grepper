use anyhow::{Context, Result};
use libloading::{Symbol, Library};

#[cfg(all(unix, not(target_os = "macos")))]
const DYLIB_EXTENSION: &str = "so";

#[cfg(windows)]
const DYLIB_EXTENSION: &str = "dll";

#[cfg(all(unix, target_os = "macos"))]
const DYLIB_EXTENSION: &str = "dylib";

#[derive(Debug)]
pub struct Language {
    inner: tree_sitter::Language,
    name: String,
}

impl Language {
    pub fn get_language(runtime_path: &std::path::Path, name: &str) -> Result<Self> {
        let name = name.to_ascii_lowercase();
        let mut library_path = runtime_path.join(&name);
        library_path.set_extension(DYLIB_EXTENSION);

        let library = unsafe { Library::new(&library_path) }
            .with_context(|| format!("Error opening dynamic library {:?}", &library_path))?;
        let language_fn_name = format!("tree_sitter_{}", name.replace('-', "_"));
        let language = unsafe {
            let language_fn: Symbol<unsafe extern "C" fn() -> tree_sitter::Language> = library
                .get(language_fn_name.as_bytes())
                .with_context(|| format!("Failed to load symbol {}", language_fn_name))?;
            language_fn()
        };
        std::mem::forget(library);
        Ok(Self {
            name,
            inner: language,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_for_types_builder(&self) -> &'static str {
        match self.name.as_str() {
            "c" => "c",
            "cpp" => "cpp",
            "elixir" => "elixir",
            "elm" => "elm",
            "go" => "go",
            "haskell" => "haskell",
            "java" => "java",
            "javascript" => "js",
            "markdown" => "markdown",
            "nix" => "nix",
            "php" => "php",
            "python" => "py",
            "ruby" => "ruby",
            "rust" => "rust",
            "typescript" => "ts",
            _ => panic!("Unknown language: {}", self.name),
        }
    }

    pub fn ts_lang(&self) -> tree_sitter::Language {
        self.inner
    }
}

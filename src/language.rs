use anyhow::{anyhow, bail, Error, Result};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use paste::paste;

macro_rules! include_langs {
    ($($lang:ident $nametb:literal),+) => {
        
        paste! {
            #[derive(PartialEq, Eq, Hash, Debug)]
            pub enum Language {
                $($lang),+
            }

            impl Language {
                pub fn all() -> Vec<Language> {
                    vec![
                        $(Language::$lang),+
                    ]
                }

                pub fn language(&self) -> tree_sitter::Language {
                    unsafe {
                        match self {
                            $(Language::$lang => [<tree_sitter_ $lang:lower>](),)+
                        }
                    }
                }

                pub fn parse_query(&self, raw: &str) -> Result<tree_sitter::Query> {
                    tree_sitter::Query::new(self.language(), raw).map_err(|err| anyhow!("{}", err))
                }

                pub fn name_for_types_builder(&self) -> &str {
                    match self {
                        $(Language::$lang => $nametb),+
                    }
                }
            }

            impl FromStr for Language {
                type Err = Error;

                fn from_str(s: &str) -> Result<Self> {
                    match s {
                        $($nametb => Ok(Language::$lang),)+
                        _ => bail!(
                            "unknown language {}. Try one of: {}",
                            s,
                            Language::all()
                                .into_iter()
                                .map(|l| l.to_string())
                                .inspect(|l| { dbg!(l);})
                                .collect::<Vec<String>>()
                                .join(", ")
                        ),
                    }
                }
            }

            impl Display for Language {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
                    match self {
                        $(Language::$lang => f.write_str(stringify!([<$lang:lower>]))),+
                    }
                }
            }

            extern "C" {
                $(fn [<tree_sitter_ $lang:lower>]() -> tree_sitter::Language;)+
            }
        }
    };
}

include_langs!(Cpp "cpp", Rust "rust", C "c", Python "py", JavaScript "js", Lua "lua", Markdown "md", Go "go");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_str_reflects_from_str() {
        // Note: this will hide results if there are multiple failures. It's
        // something that could be worked around but I don't think it is right
        // now. If it bothers you in the future, feel free to take a stab at it!
        Language::all()
            .into_iter()
            .for_each(|lang| assert_eq!(Language::from_str(&lang.to_string()).unwrap(), lang))
    }

    #[test]
    fn parse_query_smoke_test() {
        assert!(Language::Elm.parse_query("(_)").is_ok());
    }

    #[test]
    fn parse_query_problem() {
        // tree-grepper 1.0 just printed the error struct when problems like
        // this happened. This test is just here to make sure we take a slightly
        // friendlier approach for 2.0.
        assert_eq!(
            String::from("Query error at 1:2. Invalid node type node_that_doesnt_exist"),
            Language::Elm
                .parse_query("(node_that_doesnt_exist)")
                .unwrap_err()
                .to_string(),
        )
    }
}

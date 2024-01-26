use crate::language::Language;
use anyhow::{Context, Result};
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Point, Query, QueryCursor};

#[derive(Debug)]
pub struct Extractor {
    language: Language,
    ts_language: tree_sitter::Language,
    query: Query,
    captures: Vec<String>,
}

impl Extractor {
    pub fn new(language: Language, mut query: Query) -> Extractor {
        let captures = query.capture_names().to_vec();

        captures.iter().for_each(|name| {
            if name.starts_with('_') {
                query.disable_capture(name);
            }
        });

        Extractor {
            ts_language: language.ts_lang(),
            language,
            query,
            captures,
        }
    }

    pub fn language(&self) -> &Language {
        &self.language
    }

    pub fn extract_from_file(
        &self,
        path: &Path,
        parser: &mut Parser,
    ) -> Result<Option<ExtractedFile>> {
        let source = fs::read(&path).context("could not read file")?;

        self.extract_from_text(Some(path), &source, parser)
    }

    pub fn extract_from_text(
        &self,
        path: Option<&Path>,
        source: &[u8],
        parser: &mut Parser,
    ) -> Result<Option<ExtractedFile>> {
        parser
            .set_language(self.ts_language)
            .context("could not set language")?;

        let tree = parser
            .parse(&source, None)
            // note: this could be a timeout or cancellation, but we don't set
            // that so we know it's always a language error. Buuuut we also
            // always set the language above so if this happens we also know
            // it's an internal error.
            .context(
                "could not parse to a tree. This is an internal error and should be reported.",
            )?;

        let mut cursor = QueryCursor::new();

        let extracted_matches = cursor
            .matches(&self.query, tree.root_node(), source)
            .flat_map(|query_match| query_match.captures)
            // note: the casts here could potentially break if run on a 16-bit
            // microcontroller. I don't think this is a huge problem, though,
            // since even the gnarliest queries I've written have something on
            // the order of 20 matches. Nowhere close to 2^16!
            .map(|capture| {
                let name = &self.captures[capture.index as usize];
                let node = capture.node;
                let text = match node
                    .utf8_text(source)
                    .map(|unowned| unowned.to_string())
                    .context("could not extract text from capture")
                {
                    Ok(text) => text,
                    Err(problem) => return Err(problem),
                };

                Ok(ExtractedMatch {
                    kind: node.kind(),
                    name,
                    text,
                    start: node.start_position(),
                    end: node.end_position(),
                })
            })
            .collect::<Result<Vec<ExtractedMatch>>>()?;

        if extracted_matches.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ExtractedFile {
                file: path.map(|p| p.to_owned()),
                file_type: self.language.name().to_string(),
                matches: extracted_matches,
            }))
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtractedFile<'query> {
    pub(crate) file: Option<PathBuf>,
    pub(crate) file_type: String,
    pub(crate) matches: Vec<ExtractedMatch<'query>>,
}

impl<'query> Display for ExtractedFile<'query> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: is there a better way to do this unwrapping? This implementation
        // turns non-UTF-8 paths into "NON-UTF8 FILENAME". I don't know exactly
        // what circumstances that could happen in... maybe we should just wait
        // for bug reports?
        let filename = self
            .file
            .as_ref()
            .map(|f| f.to_str().unwrap_or("NON-UTF8 FILENAME"))
            .unwrap_or("NO FILE");

        for extraction in &self.matches {
            writeln!(
                f,
                "{}:{}:{}:{}:{}",
                filename,
                extraction.start.row + 1,
                extraction.start.column + 1,
                extraction.name,
                extraction.text
            )?
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtractedMatch<'query> {
    kind: &'static str,
    name: &'query str,
    text: String,
    #[serde(serialize_with = "serialize_point")]
    pub(crate) start: Point,
    #[serde(serialize_with = "serialize_point")]
    pub(crate) end: Point,
}

fn serialize_point<S>(point: &Point, sz: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut out = sz.serialize_struct("Point", 2)?;
    out.serialize_field("row", &(point.row + 1))?;
    out.serialize_field("column", &(point.column + 1))?;
    out.end()
}

use super::*;
use std::iter::FromIterator;
use std::fmt::{Display, Formatter, Debug};
use std::fmt::Result as FmtResult;

/// A structure that contains Snoot errors for easy sorting and printing
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    /// Constructs a new ErrorBag
    pub fn new() -> DiagnosticBag {
        DiagnosticBag { diagnostics: vec![] }
    }

    /// Costructs a diagnostic bag from a Vec
    pub fn from_vec(v: Vec<Diagnostic>) -> DiagnosticBag {
        DiagnosticBag { diagnostics: v }
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Sorts the errors contained in the bag for better printing.
    ///
    /// The sort order is by filename (primary) and by file location (secondary)
    pub fn sort(&mut self) {
        use std::cmp::Ord;
        self.diagnostics
            .sort_by(|e1, e2| e1.global_span.cmp(&e2.global_span));
        self.diagnostics
            .sort_by(|e1, e2| e1.global_span.file.cmp(&e2.global_span.file));
    }

    /// Appends another ErrorBag onto this one.
    pub fn append(&mut self, mut other: DiagnosticBag) {
        self.diagnostics.append(&mut other.diagnostics);
    }

    /// Adds a new error to the bag.
    pub fn add(&mut self, error: Diagnostic) {
        self.diagnostics.push(error);
    }

    /// Returns true if the bag contains any error with error level "Error"
    pub fn contains_errors(&self) -> bool {
        for error in &self.diagnostics {
            if error.error_level == DiagnosticLevel::Error {
                return true;
            }
        }
        false
    }

    /// Returns true if the bag contains any error with error level "Warn"
    pub fn contains_warnings(&self) -> bool {
        for error in &self.diagnostics {
            if error.error_level == DiagnosticLevel::Warn {
                return true;
            }
        }
        false
    }

    /// Returns true if the bag contains any error with error level "Info"
    pub fn contains_info(&self) -> bool {
        for error in &self.diagnostics {
            if error.error_level == DiagnosticLevel::Info {
                return true;
            }
        }
        false
    }

    /// Returns true if the bag contains any error with a custom error level
    pub fn contains_any_custom(&self) -> bool {
        for error in &self.diagnostics {
            if let &DiagnosticLevel::Custom(_) = &error.error_level {
                return true;
            }
        }
        false
    }

    /// Returns true if the bag contains any error with the specified custom
    /// error level.
    pub fn contains_custom(&self, custom: &str) -> bool {
        for error in &self.diagnostics {
            if let &DiagnosticLevel::Custom(ref c) = &error.error_level {
                if c == custom {
                    return true;
                }
            }
        }
        false
    }

    /// Returns true if there are no diagnostics
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// If the bag isn't empty, this will panic with the diagnostic
    /// messages as the panic string.
    pub fn assert_empty(&self) {
        if !self.is_empty() {
            panic!("{}", self);
        }
    }

    /// If the bag contains any errors, this will panic with the diagnostic
    /// messages as the panic string.
    pub fn assert_no_errors(&self) {
        if self.contains_errors() {
            panic!("{}", self);
        }
    }

    /// If the bag contains any warnings, this will panic with the diagnostic
    /// messages as the panic string.
    pub fn assert_no_warnings(&self) {
        if self.contains_warnings() {
            panic!("{}", self);
        }
    }

    /*  severity: DiagnosticSeverity.Warning,
        range: {
            start: { line: i, character: index},
            end: { line: i, character: index + 10 }
        },
        message: `${line.substr(index, 10)} should be spelled TypeScript`,
        source: 'ex'
    */
    pub fn to_json(&self) -> ::serde_json::Value {
        use serde_json::Value;

        let mut all = vec![];
        for diagnostic in &self.diagnostics {
            let sev = match diagnostic.error_level {
                DiagnosticLevel::Error => 0,
                DiagnosticLevel::Warn => 1,
                DiagnosticLevel::Info => 2,
                DiagnosticLevel::Custom(_) => 3,
            };

            let map = json!({
                "severity": sev,
                "message": diagnostic.message,
                "source": "implicit lint",
                "range": {
                    "start": {
                        "line": diagnostic.global_span.lines_covered.start - 1,
                        "character": diagnostic.global_span.columns.start - 1,
                    },
                    "end": {
                        "line": diagnostic.global_span.lines_covered.end - 1,
                        "character": diagnostic.global_span.columns.end - 1,
                    },
                }
            });
            all.push(map);
        }

        Value::Array(all)
    }

    pub fn iter(&self) -> ::std::slice::Iter<Diagnostic> {
        self.diagnostics.iter()
    }
}

impl FromIterator<Diagnostic> for DiagnosticBag {
    fn from_iter<T>(iter: T) -> Self
        where T: IntoIterator<Item = Diagnostic>
    {
        DiagnosticBag { diagnostics: iter.into_iter().collect() }
    }
}

impl Display for DiagnosticBag {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        for error in &self.diagnostics {
            writeln!(formatter, "{}", error)?;
        }
        Ok(())
    }
}

impl Debug for DiagnosticBag {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "{}", self)
    }
}

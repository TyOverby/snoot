use super::token::{ListType, TokenInfo};
use super::parse::{Span, SexprKind};
use super::diagnostic::DiagnosticBag;
use tendril::StrTendril;

/// The S-Expression tree type.
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Sexpr {
    /// An S-Expression List.
    ///
    /// Typically looks like `(...)`, `[...]`, or `{...}`.
    List {
        list_type: ListType,
        opening_token: TokenInfo,
        closing_token: TokenInfo,

        children: Vec<Sexpr>,
        span: Span,
    },

    /// An s-expression unary operator (currently impossible to construct)
    UnaryOperator {
        op: TokenInfo,
        child: Box<Sexpr>,
        span: Span,
    },

    /// A "terminal" node in the tree.
    ///
    /// Examples: `5.0`, `foo`, `asdlkh23y823ysd`.
    Terminal(TokenInfo, Span),

    /// A "string" node in the tree.
    ///
    /// Examples: `"foo"`
    String(TokenInfo, Span),
}

impl Sexpr {
    /// Returns the text that built this s-expression
    ///
    /// This is a shortcut for `.span().text()`.
    pub fn text(&self) -> StrTendril {
        self.span().text()
    }

    /// Returns an easily-matchable `SexprKind` value.
    pub fn kind(&self) -> SexprKind {
        match self {
            &Sexpr::List { .. } => SexprKind::List,
            &Sexpr::UnaryOperator { .. } => SexprKind::UnaryOperator,
            &Sexpr::String(_, _) => SexprKind::String,
            &Sexpr::Terminal(_, _) => SexprKind::Terminal,
        }
    }

    /// Returns the span over the source code that this s-expression encompasses
    pub fn span(&self) -> &Span {
        match self {
            &Sexpr::List { ref span, .. } => span,
            &Sexpr::UnaryOperator { ref span, .. } => span,
            &Sexpr::String(_, ref span) |
            &Sexpr::Terminal(_, ref span) => span,
        }
    }

    /// Returns the last token that contributed to building this expression
    pub fn last_token(&self) -> &TokenInfo {
        match self {
            &Sexpr::List { ref closing_token, .. } => closing_token,
            &Sexpr::UnaryOperator { ref child, .. } => child.last_token(),
            &Sexpr::String(ref token, _) |
            &Sexpr::Terminal(ref token, _) => token,
        }
    }

    /// Returns the first token that contrtbuted to building this expression
    pub fn first_token(&self) -> &TokenInfo {
        match self {
            &Sexpr::List { ref opening_token, .. } => opening_token,
            &Sexpr::UnaryOperator { ref op, .. } => op,
            &Sexpr::String(ref token, _) |
            &Sexpr::Terminal(ref token, _) => token,
        }
    }

    pub fn expect_int(&self, diagnostics: &mut DiagnosticBag) -> Option<i64> {
        if let &Sexpr::Terminal(_, ref span) = self {
            if let Ok(parsed) = span.text().as_ref().parse() {
                Some(parsed)
            } else {
                diagnostics
                    .add(diagnostic!(span, "Expected integer, failed to parse `{}`", span.text()));
                None
            }
        } else {
            diagnostics.add(diagnostic!(self.span(), "Expected to find an integer, but found {:?} instead", self.kind()));
            None
        }
    }

    pub fn expect_float(&self, diagnostics: &mut DiagnosticBag) -> Option<f64> {
        if let &Sexpr::Terminal(_, ref span) = self {
            if let Ok(parsed) = span.text().as_ref().parse() {
                Some(parsed)
            } else {
                diagnostics
                    .add(diagnostic!(span, "Expected number, failed to parse `{}`", span.text()));
                None
            }
        } else {
            diagnostics.add(diagnostic!(self.span(), "Expected to find a number, but found {:?} instead", self.kind()));
            None
        }
    }

    pub fn expect_list(&self, diagnostics: &mut DiagnosticBag) -> Option<&[Sexpr]> {
        if let &Sexpr::List { ref children, .. } = self {
            Some(children)
        } else {
            diagnostics.add(diagnostic!(self.span(), "Expected to find a list, but found {:?} instead", self.kind()));
            None
        }
    }

    pub fn expect_terminal(&self, symbol: &str, diagnostics: &mut DiagnosticBag) -> Option<()> {
        if let &Sexpr::Terminal(_, ref span) = self {
            if symbol == span.text().as_ref() {
                Some(())
            } else {
                diagnostics.add(diagnostic!(span, "Expected terminal `{}` found `{}`", symbol, span.text()));
                None
            }
        } else {
            diagnostics.add(diagnostic!(self.span(), "Expected terminal `{}`", symbol));
            None
        }
    }

    pub fn expect_list_with_symbol(&self,
                                   symbol: &str,
                                   diagnostics: &mut DiagnosticBag)
                                   -> Option<&[Sexpr]> {
        if let &Sexpr::List { ref children, .. } = self {
            if children.len() == 0 {
                diagnostics.add(diagnostic!(self.span(), "Expected a list with symbol `{}` but found an empty list", symbol));
                None
            } else {
                children[0]
                    .expect_terminal(symbol, diagnostics)
                    .map(|_| &children[1..])
            }
        } else {
            diagnostics.add(diagnostic!(self.span(), "Expected to find a list, but found {:?} instead", self.kind()));
            None
        }
    }
}

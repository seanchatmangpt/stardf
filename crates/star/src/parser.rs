
//! Turtle-star subset parser.
//!
//! Grammar (RDF 1.2 Turtle-star subset, nesting one level per the dispatch):
//! ```text
//! doc      := (directive | statement)*
//! directive:= '@prefix' PNAME_NS '<' IRI '>'
//! statement:= (subject predicateObjectList) '.'
//! subject  := iri | blank | quoted
//! object   := iri | blank | literal | quoted
//! quoted   := '<<' subject ws predicate ws object '>>'
//! predicate:= iri
//! ```
//! Quoted triples may appear in subject and object position; one nested
//! quoted level is accepted inside a quoted triple. Comments (`#`) and
//! whitespace separate tokens.

use crate::store::Store;
use crate::term::{Term, Triple};

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Iri(String),
    PName(String),   // "ex:foo" or "ex:" (prefix declaration value)
    Blank(String),   // _:b1
    Lit { value: String, lang: Option<String>, datatype: Option<String> },
    QuoteOpen,       // <<
    QuoteClose,      // >>
    Dot,
    PrefixKw,        // @prefix
}

fn tokenize(input: &str) -> Result<Vec<Tok>, String> {
    let b: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    let mut out = Vec::new();
    while i < b.len() {
        let c = b[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '#' {
            while i < b.len() && b[i] != '\n' { i += 1; }
            continue;
        }
        if c == '.' && (i + 1 >= b.len() || b[i + 1].is_whitespace()) {
            out.push(Tok::Dot);
            i += 1;
            continue;
        }
        if c == '<' && i + 1 < b.len() && b[i + 1] == '<' {
            out.push(Tok::QuoteOpen);
            i += 2;
            continue;
        }
        if c == '<' {
            let start = i + 1;
            let mut j = start;
            while j < b.len() && b[j] != '>' { j += 1; }
            if j >= b.len() { return Err("unterminated IRI".into()); }
            out.push(Tok::Iri(b[start..j].iter().collect()));
            i = j + 1;
            continue;
        }
        if c == '>' && i + 1 < b.len() && b[i + 1] == '>' {
            out.push(Tok::QuoteClose);
            i += 2;
            continue;
        }
        if c == '>' {
            return Err(format!("unexpected '>' at char {i}"));
        }
        if c == '"' {
            let start = i + 1;
            let mut j = start;
            while j < b.len() && b[j] != '"' {
                if b[j] == '\\' { j += 1; }
                j += 1;
            }
            if j >= b.len() { return Err("unterminated literal".into()); }
            let value: String = b[start..j].iter().collect();
            i = j + 1;
            // language tag or datatype
            let mut lang = None;
            let mut datatype = None;
            if i < b.len() && b[i] == '@' {
                let s = i + 1;
                let mut k = s;
                while k < b.len() && (b[k].is_alphanumeric() || b[k] == '-') { k += 1; }
                lang = Some(b[s..k].iter().collect());
                i = k;
            } else if i + 1 < b.len() && b[i] == '^' && b[i + 1] == '^' {
                i += 2;
                match tokenize_one_iri(&b, &mut i)? {
                    t => datatype = Some(t),
                }
            }
            out.push(Tok::Lit { value, lang, datatype });
            continue;
        }
        if c == '@' {
            let word: String = b[i + 1..].iter().take_while(|c| c.is_alphabetic()).collect();
            if word == "prefix" {
                out.push(Tok::PrefixKw);
                i += 1 + word.len();
                continue;
            }
            return Err(format!("unknown directive @{}", word));
        }
        if (c.is_alphanumeric() || c == '_') && c != ':' {
            let word: String = b[i..].iter().take_while(|c| {
                c.is_alphanumeric() || **c == '_' || **c == '-' || **c == ':'
            }).collect();
            if let Some(rest) = word.strip_prefix("_:") {
                out.push(Tok::Blank(rest.to_string()));
                i += word.chars().count();
                continue;
            }
            out.push(Tok::PName(word.clone()));
            i += word.chars().count();
            continue;
        }
        return Err(format!("unexpected char {:?} at {}", c, i));
    }
    Ok(out)
}

/// Helper for `^^` datatype IRIs or prefixed names after a literal.
fn tokenize_one_iri(b: &[char], i: &mut usize) -> Result<String, String> {
    if *i < b.len() && b[*i] == '<' {
        let start = *i + 1;
        let mut j = start;
        while j < b.len() && b[j] != '>' { j += 1; }
        if j >= b.len() { return Err("unterminated datatype IRI".into()); }
        let v: String = b[start..j].iter().collect();
        *i = j + 1;
        return Ok(v);
    }
    let word: String = b[*i..].iter().take_while(|c| {
        c.is_alphanumeric() || **c == '_' || **c == '-' || **c == ':'
    }).collect();
    *i += word.chars().count();
    Ok(word)
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
    prefixes: Vec<(String, String)>,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }
    fn next(&mut self) -> Result<Tok, String> {
        let t = self.toks.get(self.pos).cloned().ok_or("unexpected end of input")?;
        self.pos += 1;
        Ok(t)
    }
    fn resolve(&self, name: &str) -> String {
        if let Some((prefix, suffix)) = name.split_once(':') {
            if let Some((_, base)) = self.prefixes.iter().find(|(p, _)| *p == prefix) {
                return format!("{}{}", base, suffix);
            }
        }
        name.to_string()
    }

    /// Parse a subject-or-object term; `depth` counts nesting level.
    fn term(&mut self, store: &mut Store, depth: u32) -> Result<Term, String> {
        match self.next()? {
            Tok::Iri(v) => Ok(Term::Iri(self.resolve(&v))),
            Tok::PName(v) => Ok(Term::Iri(self.resolve(&v))),
            Tok::Blank(v) => Ok(Term::Blank(v)),
            Tok::Lit { value, lang, datatype } => {
                let datatype = datatype.map(|d| self.resolve(&d));
                Ok(Term::Literal { value, lang, datatype })
            }
            Tok::QuoteOpen => {
                if depth >= 2 {
                    return Err("quoted-triple nesting deeper than one level".into());
                }
                let s = self.term(store, depth + 1)?;
                let p = match self.next()? {
                    Tok::Iri(v) => self.resolve(&v),
                    Tok::PName(v) => self.resolve(&v),
                    other => return Err(format!("quoted predicate: {:?}", other)),
                };
                let o = self.term(store, depth + 1)?;
                if self.next()? != Tok::QuoteClose {
                    return Err("missing >> closing a quoted triple".into());
                }
                Ok(store.quote(s, &p, o))
            }
            other => Err(format!("unexpected token in term position: {:?}", other)),
        }
    }
}

/// Parse a Turtle-star subset document into `store`; returns the number of
/// top-level statements added.
pub fn parse(input: &str, store: &mut Store) -> Result<usize, String> {
    let toks = tokenize(input)?;
    let mut p = Parser { toks, pos: 0, prefixes: Vec::new() };
    let mut count = 0usize;
    while p.peek().is_some() {
        match p.next()? {
            Tok::PrefixKw => {
                let pname = match p.next()? {
                    Tok::PName(v) => v,
                    other => return Err(format!("prefix name: {:?}", other)),
                };
                let prefix = pname.strip_suffix(':').ok_or("prefix must end with ':'")?.to_string();
                let iri = match p.next()? {
                    Tok::Iri(v) => v,
                    other => return Err(format!("prefix IRI: {:?}", other)),
                };
                p.prefixes.push((prefix, iri));
                // Turtle '@prefix' directives end with a dot.
                if p.next()? != Tok::Dot {
                    return Err("prefix directive missing '.'".into());
                }
            }
            Tok::Iri(_) | Tok::PName(_) | Tok::Blank(_) | Tok::QuoteOpen => {
                p.pos -= 1;
                let s = p.term(store, 0)?;
                let mut pairs: Vec<(String, Term)> = Vec::new();
                loop {
                    let pred = match p.next()? {
                        Tok::Iri(v) => p.resolve(&v),
                        Tok::PName(v) => p.resolve(&v),
                        other => return Err(format!("predicate: {:?}", other)),
                    };
                    let obj = p.term(store, 0)?;
                    pairs.push((pred, obj));
                    match p.peek() {
                        // ';' continues the predicate-object list
                        Some(Tok::PName(v)) if v.ends_with(';') => {
                            return Err("';' lists are not in this subset; use one pred-obj per statement".into());
                        }
                        _ => break,
                    }
                }
                if p.next()? != Tok::Dot {
                    return Err("statement missing '.'".into());
                }
                for (pred, obj) in pairs {
                    store.add(Triple::new(s.clone(), &pred, obj));
                }
                count += 1;
            }
            other => return Err(format!("unexpected token at statement start: {:?}", other)),
        }
    }
    Ok(count)
}

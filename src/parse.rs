use crate::Span;
use syn::parse::{Error, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parenthesized, LitStr, Token};

mod kw {
    syn::custom_keyword!(allow);
    syn::custom_keyword!(warn);
    syn::custom_keyword!(deny);
    syn::custom_keyword!(feature);
}

#[derive(Default)]
pub(crate) struct AWD {
    pub allow: Vec<(String, Span)>,
    pub warn: Vec<(String, Span)>,
    pub deny: Vec<(String, Span)>,
}

impl AWD {
    pub(crate) fn is_empty(&self) -> bool {
        self.allow.is_empty() && self.warn.is_empty() && self.deny.is_empty()
    }
}

// #[allow(clippy::lint_id...)]
pub(crate) fn allow(input: ParseStream) -> Result<AWD> {
    let awd = AWD {
        allow: collect_clippy_lints(input)?,
        warn: Vec::new(),
        deny: Vec::new(),
    };
    Ok(awd)
}

pub(crate) fn warn(input: ParseStream) -> Result<AWD> {
    let awd = AWD {
        allow: Vec::new(),
        warn: collect_clippy_lints(input)?,
        deny: Vec::new(),
    };
    Ok(awd)
}

pub(crate) fn deny(input: ParseStream) -> Result<AWD> {
    let awd = AWD {
        allow: Vec::new(),
        warn: Vec::new(),
        deny: collect_clippy_lints(input)?,
    };
    Ok(awd)
}

fn collect_clippy_lints(input: ParseStream) -> Result<Vec<(String, Span)>> {
    let paths = Punctuated::<syn::Path, Token![,]>::parse_terminated(input)?;

    let mut lints = Vec::new();
    for path in paths {
        if path.segments.len() == 2 && path.segments[0].ident == "clippy" {
            let clippy_ident = &path.segments[0].ident;
            let lint_ident = &path.segments[1].ident;
            let span = Span {
                start: clippy_ident.span().start(),
                end: lint_ident.span().end(),
            };
            lints.push((lint_ident.to_string(), span));
        }
    }

    Ok(lints)
}

// #[cfg_attr(feature = "cargo-clippy", allow(lint_id...))]
pub(crate) fn cfg_attr(input: ParseStream) -> Result<AWD> {
    input.parse::<kw::feature>()?;
    input.parse::<Token![=]>()?;
    let feature = input.parse::<LitStr>()?;
    if feature.value() != "cargo-clippy" {
        let msg = "expected feature = \"cargo-clippy\"";
        return Err(Error::new(feature.span(), msg));
    }

    let mut awd = AWD {
        allow: Vec::new(),
        warn: Vec::new(),
        deny: Vec::new(),
    };

    macro_rules! or_return {
        ($e:expr) => {
            match $e {
                Ok(x) => x,
                Err(_) => {
                    return Ok(awd);
                }
            }
        };
    }

    or_return!(input.parse::<Token![,]>());

    loop {
        if input.peek(kw::allow) {
            or_return!(input.parse::<kw::allow>());
            awd.allow.extend(or_return!(cfg_attr_inner(input)));
        } else if input.peek(kw::warn) {
            or_return!(input.parse::<kw::warn>());
            awd.warn.extend(or_return!(cfg_attr_inner(input)));
        } else if input.peek(kw::deny) {
            or_return!(input.parse::<kw::deny>());
            awd.deny.extend(or_return!(cfg_attr_inner(input)));
        } else {
            return Ok(awd);
        }
    }
}

fn cfg_attr_inner(input: ParseStream) -> Result<Vec<(String, Span)>> {
    let list;
    parenthesized!(list in input);
    input.parse::<Option<Token![,]>>()?;

    let paths = Punctuated::<syn::Path, Token![,]>::parse_terminated(&list)?;

    let mut lints = Vec::new();
    for path in paths {
        if let Some(lint_ident) = path.get_ident() {
            let span = Span {
                start: lint_ident.span().start(),
                end: lint_ident.span().end(),
            };
            lints.push((lint_ident.to_string(), span));
        }
    }

    Ok(lints)
}

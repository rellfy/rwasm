extern crate proc_macro;
use proc_macro::{Ident, TokenStream, TokenTree};

use std::iter::Peekable;

fn next_group(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<proc_macro::Group> {
    if let Some(TokenTree::Group(_)) = source.peek() {
        let group = match source.next().unwrap() {
            TokenTree::Group(group) => group,
            _ => unreachable!("just checked with peek()!"),
        };
        Some(group)
    } else {
        None
    }
}

fn next_literal(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<String> {
    if let Some(TokenTree::Literal(lit)) = source.peek() {
        let mut literal = lit.to_string();

        // the only way to check that literal is string :/
        if literal.starts_with("\"") {
            literal.remove(0);
            literal.remove(literal.len() - 1);
        }
        source.next();
        return Some(literal);
    }

    return None;
}

#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut modified = TokenStream::new();
    let mut source = item.into_iter().peekable();

    let mut crate_rename = None;

    while let Some(TokenTree::Punct(punct)) = source.peek() {
        assert_eq!(format!("{}", punct), "#");

        // skip '#'
        let _ = source.next().unwrap();

        let group = next_group(&mut source);
        let mut group = group.unwrap().stream().into_iter().peekable();
        let attribute_name = format!("{}", group.next().unwrap());

        // skipping non-relevant attributes
        if attribute_name == "rwasm" {
            let group = next_group(&mut group);
            let mut group = group.unwrap().stream().into_iter().peekable();
            let config_name = format!("{}", group.next().unwrap());

            if group.peek().is_some() {
                // skip '='
                let _ = group.next();

                let config_value = Some(next_literal(&mut group).unwrap());

                if config_name == "crate_rename" {
                    crate_rename = config_value;
                }
            }
        }
    }

    if let TokenTree::Ident(ident) = source.next().unwrap() {
        assert_eq!(format!("{}", ident), "async");

        modified.extend(std::iter::once(TokenTree::Ident(ident)));
    } else {
        panic!("[rwasm::main] is allowed only for async functions");
    }

    if let TokenTree::Ident(ident) = source.next().unwrap() {
        assert_eq!(format!("{}", ident), "fn");

        modified.extend(std::iter::once(TokenTree::Ident(ident)));
    } else {
        panic!("[rwasm::main] is allowed only for functions");
    }

    if let TokenTree::Ident(ident) = source.next().unwrap() {
        assert_eq!(format!("{}", ident), "main");

        modified.extend(std::iter::once(TokenTree::Ident(Ident::new(
            "amain",
            ident.span(),
        ))));
    } else {
        panic!("[rwasm::main] expecting main function");
    }

    modified.extend(source);

    let mut prelude: TokenStream = format!(
        "
        fn main() {{
            {crate_name}::wasync::Executor::spawn(amain());
        }}
        ",
        crate_name = crate_rename.unwrap_or_else(|| "rwasm".to_string()),
    )
    .parse()
    .unwrap();
    prelude.extend(modified);

    prelude
}

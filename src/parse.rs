use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    braced,
    ext::IdentExt as _,
    parse::{discouraged::Speculative, Parse, ParseStream},
    punctuated::Punctuated,
    token, Ident, LitStr, Result, Token,
};

use crate::ast::{
    Attr, ClosingTag, DashIdent, Doctype, Element, Node, NodeTree,
    OpeningOrVoidTag, OpeningTag, Value, VoidTag,
};

pub fn parse(input: TokenStream) -> Result<Box<[Node]>> {
    Ok(syn::parse::<NodeTree>(input)?.0)
}

pub fn parse2(input: TokenStream2) -> Result<Box<[Node]>> {
    Ok(syn::parse2::<NodeTree>(input)?.0)
}

impl Parse for DashIdent {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse a non-empty sequence of identifiers separated by dashes.
        let inner =
            Punctuated::<Ident, Token![-]>::parse_separated_nonempty_with(
                input,
                Ident::parse_any,
            )?;

        Ok(Self(inner))
    }
}

impl Parse for Doctype {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            lt_sign: input.parse()?,
            excl_mark: input.parse()?,
            doctype: input.parse()?,
            html: input.parse()?,
            gt_sign: input.parse()?,
        })
    }
}

impl Parse for Value {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            Ok(Self::LitStr(input.parse()?))
        } else if lookahead.peek(token::Brace) {
            let content;
            braced!(content in input);
            Ok(Self::Expr(content.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitStr) | input.peek(token::Brace) {
            Ok(Self::Value(input.parse()?))
        } else {
            Ok(Self::Keyed {
                key: input.parse()?,
                eq_sign: input.parse()?,
                value: input.parse()?,
            })
        }
    }
}

impl Parse for OpeningTag {
    fn parse(input: ParseStream) -> Result<Self> {
        let lt_sign = input.parse()?;
        let name = input.parse()?;

        let mut attrs = Vec::new();
        while !(input.peek(Token![>])) {
            attrs.push(input.parse()?);
        }

        let gt_sign = input.parse()?;

        Ok(Self {
            lt_sign,
            name,
            attrs,
            gt_sign,
        })
    }
}

impl Parse for ClosingTag {
    fn parse(input: ParseStream) -> Result<Self> {
        let lt_sign = input.parse()?;
        let slash = input.parse()?;
        let name = input.parse()?;
        let gt_sign = input.parse()?;

        Ok(Self {
            lt_sign,
            slash,
            name,
            gt_sign,
        })
    }
}

impl Parse for OpeningOrVoidTag {
    fn parse(input: ParseStream) -> Result<Self> {
        let lt_sign = input.parse()?;
        let name = input.parse()?;

        let mut attrs = Vec::new();
        while !(input.peek(Token![>])
            || (input.peek(Token![/]) && input.peek2(Token![>])))
        {
            attrs.push(input.parse()?);
        }

        if let Some(void_slash) = input.parse::<Option<Token![/]>>()? {
            let gt_sign = input.parse()?;

            return Ok(Self::Void(VoidTag {
                lt_sign,
                name,
                attrs,
                void_slash,
                gt_sign,
            }));
        }

        let gt_sign = input.parse()?;

        Ok(Self::Opening(OpeningTag {
            lt_sign,
            name,
            attrs,
            gt_sign,
        }))
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let tag = input.parse::<OpeningOrVoidTag>()?;

        match tag {
            OpeningOrVoidTag::Opening(opening_tag) => {
                let mut nodes = Vec::new();

                let closing_tag = loop {
                    let fork = input.fork();

                    if let Ok(closing_tag) = fork.parse::<ClosingTag>() {
                        if closing_tag.name == opening_tag.name {
                            input.advance_to(&fork);
                            break closing_tag;
                        }
                    };

                    nodes.push(input.parse()?);
                };

                Ok(Self::OpeningClosing {
                    opening_tag,
                    children: nodes,
                    closing_tag,
                })
            }
            OpeningOrVoidTag::Void(void_tag) => Ok(Self::Void(void_tag)),
        }
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![<]) {
            Ok(Node::Element(input.parse()?))
        } else if lookahead.peek(LitStr) || lookahead.peek(token::Brace) {
            Ok(Node::Value(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for NodeTree {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut res = Vec::new();
        while !input.is_empty() {
            res.push(input.parse()?);
        }
        Ok(Self(res.into_boxed_slice()))
    }
}

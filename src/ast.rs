use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{punctuated::Punctuated, Expr, Ident, LitStr, Token};

pub(crate) mod kw {
    use syn::custom_keyword;

    custom_keyword!(DOCTYPE);
    custom_keyword!(html);
}

/// An identifier seperated by dashes: `foo-bar-baz`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashIdent(pub Punctuated<Ident, Token![-]>);

/// An HTML doctype declaration: <!DOCTYPE html>.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Doctype {
    pub lt_sign: Token![<],
    pub excl_mark: Token![!],
    pub doctype: kw::DOCTYPE,
    pub html: kw::html,
    pub gt_sign: Token![>],
}

/// A value, either a string literal or a braced expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A string literal: `"foo bar"`.
    LitStr(LitStr),
    /// A braced expression: `{1 + 2}`.
    Expr(Expr),
}

/// An HTML attribute consisting of a key and a value: `foo="bar"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attr {
    Value(Value),
    Keyed {
        key: DashIdent,
        eq_sign: Token![=],
        value: Value,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningTag {
    pub lt_sign: Token![<],
    pub name: DashIdent,
    pub attrs: Vec<Attr>,
    pub gt_sign: Token![>],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosingTag {
    pub lt_sign: Token![<],
    pub slash: Token![/],
    pub name: DashIdent,
    pub gt_sign: Token![>],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoidTag {
    pub lt_sign: Token![<],
    pub name: DashIdent,
    pub attrs: Vec<Attr>,
    pub void_slash: Token![/],
    pub gt_sign: Token![>],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpeningOrVoidTag {
    Opening(OpeningTag),
    Void(VoidTag),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    OpeningClosing {
        opening_tag: OpeningTag,
        children: Vec<Node>,
        closing_tag: ClosingTag,
    },
    Void(VoidTag),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Element(Element),
    Value(Value),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeTree(pub Box<[Node]>);

impl ToTokens for DashIdent {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl ToTokens for Doctype {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lt_sign.to_tokens(tokens);
        self.excl_mark.to_tokens(tokens);
        self.doctype.to_tokens(tokens);
        self.html.to_tokens(tokens);
        self.gt_sign.to_tokens(tokens);
    }
}

impl ToTokens for Value {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Value::LitStr(lit_str) => lit_str.to_tokens(tokens),
            Value::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}

impl ToTokens for Attr {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Attr::Value(value) => value.to_tokens(tokens),
            Attr::Keyed {
                key,
                eq_sign,
                value,
            } => {
                key.to_tokens(tokens);
                eq_sign.to_tokens(tokens);
                value.to_tokens(tokens);
            }
        }
    }
}

impl ToTokens for OpeningTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            lt_sign,
            name,
            attrs,
            gt_sign,
        } = self;
        lt_sign.to_tokens(tokens);
        name.to_tokens(tokens);
        for attr in attrs {
            attr.to_tokens(tokens);
        }
        gt_sign.to_tokens(tokens);
    }
}

impl ToTokens for ClosingTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            lt_sign,
            slash,
            name,
            gt_sign,
        } = self;
        lt_sign.to_tokens(tokens);
        slash.to_tokens(tokens);
        name.to_tokens(tokens);
        gt_sign.to_tokens(tokens);
    }
}

impl ToTokens for VoidTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            lt_sign,
            name,
            attrs,
            void_slash,
            gt_sign,
        } = self;
        lt_sign.to_tokens(tokens);
        name.to_tokens(tokens);
        for attr in attrs {
            attr.to_tokens(tokens);
        }
        void_slash.to_tokens(tokens);
        gt_sign.to_tokens(tokens);
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::OpeningClosing {
                opening_tag,
                children,
                closing_tag,
            } => {
                opening_tag.to_tokens(tokens);
                for child in children {
                    child.to_tokens(tokens);
                }
                closing_tag.to_tokens(tokens);
            }
            Self::Void(void_tag) => {
                void_tag.to_tokens(tokens);
            }
        }
    }
}

impl ToTokens for Node {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Node::Element(el) => el.to_tokens(tokens),
            Node::Value(value) => value.to_tokens(tokens),
        }
    }
}

impl ToTokens for NodeTree {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for node in &self.0 {
            node.to_tokens(tokens);
        }
    }
}

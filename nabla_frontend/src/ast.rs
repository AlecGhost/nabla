use crate::{
    parser,
    token::{ToTokenRange, TokenRange},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstInfo {
    pub range: TokenRange,
    pub errors: Vec<parser::Error>,
}

impl AstInfo {
    pub const fn new(range: TokenRange) -> Self {
        Self {
            range,
            errors: Vec::new(),
        }
    }

    pub fn new_with_errors(range: TokenRange, errors: Vec<parser::Error>) -> Self {
        Self { range, errors }
    }

    pub fn append_error(&mut self, error: parser::Error) {
        self.errors.push(error);
    }
}

impl ToTokenRange for AstInfo {
    fn to_token_range(&self) -> TokenRange {
        self.range.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub globals: Vec<Global>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Global {
    Use(Use),
    Def(Def),
    Let(Let),
    Init(Expr),
    GlobalError(AstInfo),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Use {
    pub use_kw: AstInfo,
    pub name: Option<Ident>,
    pub body: Option<UseBody>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseBody {
    pub double_colon: AstInfo,
    pub kind: Option<UseKind>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UseKind {
    All(AstInfo),
    Single(UseItem),
    Multiple(UseItems),
    Error(AstInfo),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseItem {
    pub name: Ident,
    pub body: Option<Box<UseBody>>,
    pub alias: Option<Alias>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseItems {
    pub lcurly: AstInfo,
    pub names: Vec<UseItem>,
    pub rcurly: Option<AstInfo>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Def {
    pub def_kw: AstInfo,
    pub name: Option<Ident>,
    pub eq: Option<AstInfo>,
    pub expr: Option<Expr>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Let {
    pub let_kw: AstInfo,
    pub name: Option<Ident>,
    pub eq: Option<AstInfo>,
    pub expr: Option<Expr>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Init {
    pub expr: Expr,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Union(Union),
    Single(Single),
    Error(AstInfo),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Union {
    pub single: Single,
    pub alternatives: Vec<UnionAlternative>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnionAlternative {
    pub pipe: AstInfo,
    pub single: Option<Single>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Single {
    Struct(Struct),
    List(List),
    Named(Named),
    Primitive(Primitive),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Struct {
    pub lcurly: AstInfo,
    pub fields: Vec<StructField>,
    pub rcurly: Option<AstInfo>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructField {
    pub name: Ident,
    pub colon: Option<AstInfo>,
    pub type_expr: Option<Expr>,
    pub eq: Option<AstInfo>,
    pub expr: Option<Expr>,
    pub alias: Option<Alias>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct List {
    pub lbracket: AstInfo,
    pub expr: Option<Box<Expr>>,
    pub rbracket: Option<AstInfo>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Named {
    pub name: Ident,
    pub inner_names: Vec<InnerName>,
    pub expr: Option<Box<Expr>>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InnerName {
    pub double_colon: AstInfo,
    pub name: Option<Ident>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Primitive {
    String(AstInfo),
    Char(AstInfo),
    Number(AstInfo),
    Bool(AstInfo), // Either token TRUE or FALSE
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alias {
    pub as_kw: AstInfo,
    pub name: Option<AliasName>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliasName {
    String(Ident), // STRING token, but holds the same information as `Ident`.
    Ident(Ident),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub info: AstInfo,
}

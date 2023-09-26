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

    pub fn join(self, other: Self) -> Self {
        let start = self.range.start.min(other.range.start);
        let end = self.range.end.max(other.range.end);
        let mut errors = self.errors;
        errors.extend(other.errors);
        Self {
            range: start..end,
            errors,
        }
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
    Error(AstInfo),
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
    pub exprs: Vec<Expr>,
    pub rbracket: Option<AstInfo>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Named {
    pub name: Ident,
    pub inner_names: Vec<InnerName>,
    pub expr: Option<StructOrList>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InnerName {
    pub double_colon: AstInfo,
    pub name: Option<Ident>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructOrList {
    Struct(Struct),
    List(List),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Primitive {
    String(PrimitiveValue),
    Char(PrimitiveValue),
    Number(PrimitiveValue),
    Bool(Bool), // Either token TRUE or FALSE
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alias {
    pub as_kw: AstInfo,
    pub name: Option<AliasName>,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliasName {
    String(PrimitiveValue),
    Ident(Ident),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub info: AstInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrimitiveValue {
    pub value: String,
    pub info: AstInfo,
}

impl PrimitiveValue {
    // clippy proposes to make the function const, but the compiler disagrees
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn new(tuple: (String, AstInfo)) -> Self {
        let (value, info) = tuple;
        Self { value, info }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bool {
    pub value: bool,
    pub info: AstInfo,
}

impl Bool {
    pub(crate) fn new_true(info: AstInfo) -> Self {
        Self { value: true, info }
    }

    pub(crate) fn new_false(info: AstInfo) -> Self {
        Self { value: false, info }
    }
}

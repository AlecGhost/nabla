use crate::ast::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub kind: NodeKind,
    pub children: Vec<Node>,
    pub value: Option<String>,
    pub info: AstInfo,
}

pub enum QueryType {
    AnyLevel,
    FirstLevel,
    DirectChildren,
}

impl Node {
    const fn childless(kind: NodeKind, info: AstInfo) -> Self {
        Self {
            kind,
            children: Vec::new(),
            value: None,
            info,
        }
    }

    fn wrapper(kind: NodeKind, child: Node) -> Self {
        Self {
            kind,
            info: child.info.clone(),
            children: vec![child],
            value: None,
        }
    }

    fn level_query(&self, kind: NodeKind, stop_after_first: bool) -> Vec<&Self> {
        let mut result = Vec::new();
        if self.kind == kind {
            result.push(self);
            if stop_after_first {
                return result;
            }
        }
        let children: Vec<&Self> = self
            .children
            .iter()
            .flat_map(|child| child.level_query(kind, stop_after_first))
            .collect();
        result.extend(children);
        result
    }

    pub fn query(&self, params: &[(NodeKind, QueryType)]) -> Vec<&Self> {
        let mut nodes: Vec<&Self> = vec![self];
        for (kind, query_type) in params {
            match query_type {
                QueryType::AnyLevel | QueryType::FirstLevel => {
                    nodes = nodes
                        .into_iter()
                        .flat_map(|node| {
                            node.level_query(*kind, matches!(query_type, QueryType::FirstLevel))
                        })
                        .collect();
                }
                QueryType::DirectChildren => {
                    nodes = nodes
                        .into_iter()
                        .flat_map(|node| &node.children)
                        .filter(|child_node| child_node.kind == *kind)
                        .collect()
                }
            }
        }
        nodes
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Program,
    GlobalError,
    Use,
    UseBody,
    UseAll,
    UseError,
    UseItem,
    UseItems,
    Def,
    Let,
    Init,
    Expr,
    ExprError,
    Single,
    Union,
    UnionAlternative,
    Struct,
    StructField,
    List,
    Named,
    InnerName,
    Alias,
    String,
    Char,
    Number,
    Bool,
    Ident,
    Primitive,
    PrimitiveValue,
    UseKw,
    DefKw,
    LetKw,
    AsKw,
    DoubleColon,
    Colon,
    Eq,
    LCurly,
    RCurly,
    LBracket,
    RBracket,
    Pipe,
}

macro_rules! push {
    ($children:expr, $kind:expr, $info:expr) => {
        $children.push(Node::childless($kind, $info));
    };
    ($children:expr, $opt:expr) => {
        if let Some(value) = $opt {
            $children.push(value.into());
        }
    };
}

impl From<&Program> for Node {
    fn from(value: &Program) -> Self {
        Self {
            kind: NodeKind::Program,
            children: value.globals.iter().map(Self::from).collect(),
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&Global> for Node {
    fn from(value: &Global) -> Self {
        match value {
            Global::Use(global) => global.into(),
            Global::Def(global) => global.into(),
            Global::Let(global) => global.into(),
            Global::Init(global) => Node::wrapper(NodeKind::Init, global.into()),
            Global::Error(info) => Self::childless(NodeKind::GlobalError, info.clone()),
        }
    }
}

impl From<&Use> for Node {
    fn from(value: &Use) -> Self {
        let mut children = Vec::new();
        push!(children, NodeKind::UseKw, value.use_kw.clone());
        push!(children, &value.name);
        push!(children, &value.body);
        Self {
            kind: NodeKind::Use,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&UseBody> for Node {
    fn from(value: &UseBody) -> Self {
        let mut children = Vec::new();
        push!(children, NodeKind::DoubleColon, value.double_colon.clone());
        push!(children, &value.kind);
        Self {
            kind: NodeKind::UseBody,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&UseKind> for Node {
    fn from(value: &UseKind) -> Self {
        match value {
            UseKind::All(info) => Self::childless(NodeKind::UseAll, info.clone()),
            UseKind::Single(item) => item.into(),
            UseKind::Multiple(items) => items.into(),
            UseKind::Error(info) => Self::childless(NodeKind::UseError, info.clone()),
        }
    }
}

impl From<&UseItem> for Node {
    fn from(value: &UseItem) -> Self {
        let mut children = Vec::new();
        children.push(Self::from(&value.name));
        if let Some(body) = &value.body {
            children.push(Self::from(body.as_ref()))
        }
        push!(children, &value.alias);
        Self {
            kind: NodeKind::UseItem,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&UseItems> for Node {
    fn from(value: &UseItems) -> Self {
        let mut children = Vec::new();
        push!(children, NodeKind::LCurly, value.lcurly.clone());
        for use_item in &value.items {
            children.push(Self::from(use_item));
        }
        if let Some(info) = &value.rcurly {
            push!(children, NodeKind::RCurly, info.clone());
        }
        Self {
            kind: NodeKind::UseItems,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&Def> for Node {
    fn from(value: &Def) -> Self {
        let mut children = Vec::new();
        push!(children, NodeKind::DefKw, value.def_kw.clone());
        push!(children, &value.name);
        if let Some(info) = &value.eq {
            push!(children, NodeKind::Eq, info.clone());
        }
        push!(children, &value.expr);
        Self {
            kind: NodeKind::Def,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&Let> for Node {
    fn from(value: &Let) -> Self {
        let mut children = Vec::new();
        push!(children, NodeKind::LetKw, value.let_kw.clone());
        push!(children, &value.name);
        if let Some(info) = &value.eq {
            push!(children, NodeKind::Eq, info.clone());
        }
        push!(children, &value.expr);
        Self {
            kind: NodeKind::Def,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&Expr> for Node {
    fn from(value: &Expr) -> Self {
        let child_node = match value {
            Expr::Union(union) => union.into(),
            Expr::Single(single) => single.into(),
            Expr::Error(info) => Self::childless(NodeKind::ExprError, info.clone()),
        };
        Node::wrapper(NodeKind::Expr, child_node)
    }
}

impl From<&Union> for Node {
    fn from(value: &Union) -> Self {
        let mut children = Vec::new();
        children.push(Self::from(&value.single));
        for union_alternative in &value.alternatives {
            children.push(Self::from(union_alternative));
        }
        Self {
            kind: NodeKind::Union,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&UnionAlternative> for Node {
    fn from(value: &UnionAlternative) -> Self {
        let mut children = Vec::new();
        push!(children, NodeKind::Pipe, value.pipe.clone());
        push!(children, &value.single);
        Self {
            kind: NodeKind::UnionAlternative,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&Single> for Node {
    fn from(value: &Single) -> Self {
        let child_node = match value {
            Single::Struct(s) => s.into(),
            Single::List(list) => list.into(),
            Single::Named(named) => named.into(),
            Single::Primitive(primitive) => primitive.into(),
        };
        Node::wrapper(NodeKind::Single, child_node)
    }
}

impl From<&Struct> for Node {
    fn from(value: &Struct) -> Self {
        let mut children = Vec::new();
        push!(children, NodeKind::LCurly, value.lcurly.clone());
        for field in &value.fields {
            children.push(Self::from(field));
        }
        if let Some(info) = &value.rcurly {
            push!(children, NodeKind::RCurly, info.clone());
        }
        Self {
            kind: NodeKind::Struct,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&StructField> for Node {
    fn from(value: &StructField) -> Self {
        let mut children = Vec::new();
        children.push(Self::from(&value.name));
        if let Some(info) = &value.colon {
            push!(children, NodeKind::Colon, info.clone());
        }
        push!(children, &value.type_expr);
        if let Some(info) = &value.eq {
            push!(children, NodeKind::Eq, info.clone());
        }
        push!(children, &value.expr);
        push!(children, &value.alias);
        Self {
            kind: NodeKind::StructField,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&StructOrList> for Node {
    fn from(value: &StructOrList) -> Self {
        match value {
            StructOrList::Struct(s) => s.into(),
            StructOrList::List(list) => list.into(),
        }
    }
}

impl From<&List> for Node {
    fn from(value: &List) -> Self {
        let mut children = Vec::new();
        push!(children, NodeKind::LBracket, value.lbracket.clone());
        for expr in &value.exprs {
            children.push(Self::from(expr));
        }
        if let Some(info) = &value.rbracket {
            push!(children, NodeKind::RBracket, info.clone());
        }
        Self {
            kind: NodeKind::List,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&Named> for Node {
    fn from(value: &Named) -> Self {
        let mut children = Vec::new();
        children.push(Self::from(&value.name));
        for inner_name in &value.inner_names {
            children.push(Self::from(inner_name));
        }
        push!(children, &value.expr);
        Self {
            kind: NodeKind::Named,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&InnerName> for Node {
    fn from(value: &InnerName) -> Self {
        let mut children = Vec::new();
        push!(children, NodeKind::DoubleColon, value.double_colon.clone());
        push!(children, &value.name);
        Self {
            kind: NodeKind::Named,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&Primitive> for Node {
    fn from(value: &Primitive) -> Self {
        let child_node = match value {
            Primitive::String(s) => Node::wrapper(NodeKind::String, s.into()),
            Primitive::Char(c) => Node::wrapper(NodeKind::Char, c.into()),
            Primitive::Number(n) => Node::wrapper(NodeKind::Number, n.into()),
            Primitive::Bool(b) => b.into(),
        };
        Node::wrapper(NodeKind::Primitive, child_node)
    }
}

impl From<&Alias> for Node {
    fn from(value: &Alias) -> Self {
        let mut children = Vec::new();
        push!(children, NodeKind::AsKw, value.as_kw.clone());
        push!(children, &value.name);
        Self {
            kind: NodeKind::Alias,
            children,
            value: None,
            info: value.info.clone(),
        }
    }
}

impl From<&AliasName> for Node {
    fn from(value: &AliasName) -> Self {
        match value {
            AliasName::String(s) => Node::wrapper(NodeKind::String, s.into()),
            AliasName::Ident(ident) => ident.into(),
        }
    }
}

impl From<&Ident> for Node {
    fn from(value: &Ident) -> Self {
        Self {
            kind: NodeKind::Ident,
            children: Vec::new(),
            value: Some(value.name.clone()),
            info: value.info.clone(),
        }
    }
}

impl From<&PrimitiveValue> for Node {
    fn from(value: &PrimitiveValue) -> Self {
        Self {
            kind: NodeKind::PrimitiveValue,
            children: Vec::new(),
            value: Some(value.value.to_string()),
            info: value.info.clone(),
        }
    }
}

impl From<&Bool> for Node {
    fn from(value: &Bool) -> Self {
        Self {
            kind: NodeKind::Bool,
            children: Vec::new(),
            value: Some(value.value.to_string()),
            info: value.info.clone(),
        }
    }
}

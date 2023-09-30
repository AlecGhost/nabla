use crate::{
    ast::Program,
    node::{Node, NodeKind, QueryType},
};

use self::error::{Error, ErrorMessage};

mod error;

pub fn analyze_structure(program: &Program) -> Vec<Error> {
    let mut errors = Vec::new();
    let program_node = Node::from(program);
    let mut query = |path, message: ErrorMessage| {
        program_node
            .query(path)
            .iter()
            .map(|node| Error::new(message.clone(), node.info.range.clone()))
            .for_each(|error| errors.push(error));
    };
    query(
        &[
            (NodeKind::Use, QueryType::DirectChildren),
            (NodeKind::UseItem, QueryType::AnyLevel),
            (NodeKind::Alias, QueryType::AnyLevel),
            (NodeKind::String, QueryType::DirectChildren),
        ],
        ErrorMessage::AliasMustBeIdent,
    );
    query(
        &[
            (NodeKind::StructField, QueryType::AnyLevel),
            (NodeKind::Alias, QueryType::FirstLevel),
            (NodeKind::Ident, QueryType::DirectChildren),
        ],
        ErrorMessage::AliasMustBeString,
    );
    query(
        &[(NodeKind::Init, QueryType::DirectChildren), (NodeKind::Union, QueryType::AnyLevel)],
        ErrorMessage::UnionInInit,
    );
    errors
}

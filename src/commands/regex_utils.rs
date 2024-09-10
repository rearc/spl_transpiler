use anyhow::Result;
use regex_syntax::ast::*;

type SimplifiedGroup = (u32, Option<String>);

fn _visit_class_set(kind: ClassSet, groups: &mut Vec<SimplifiedGroup>) {
    match kind {
        ClassSet::Item(ref class_set_item) => _visit_class_set_item(class_set_item.clone(), groups),
        ClassSet::BinaryOp(ClassSetBinaryOp {
            ref lhs, ref rhs, ..
        }) => {
            _visit_class_set(*lhs.clone(), groups);
            _visit_class_set(*rhs.clone(), groups);
        }
    }
}

fn _visit_class_set_item(class_set_item: ClassSetItem, groups: &mut Vec<SimplifiedGroup>) {
    match class_set_item {
        // ClassSetItem::Empty(_) => {}
        // ClassSetItem::Literal(_) => {}
        // ClassSetItem::Range(_) => {}
        // ClassSetItem::Ascii(_) => {}
        // ClassSetItem::Unicode(_) => {}
        // ClassSetItem::Perl(_) => {}
        ClassSetItem::Bracketed(bracketed) => {
            let ClassBracketed { kind, .. } = *bracketed;
            _visit_class_set(kind, groups)
        }
        ClassSetItem::Union(ClassSetUnion { items, .. }) => {
            for item in items {
                _visit_class_set_item(item, groups);
            }
        }
        _ => {}
    }
}

fn _visit_node(node: Ast, groups: &mut Vec<SimplifiedGroup>) {
    // https://docs.rs/regex-syntax/latest/regex_syntax/ast/enum.Ast.html
    // We only care about recursing through branches that may contain named groups
    // When we find a named group, save its numeric position and name
    match node {
        // Ast::Empty(_) => {}
        // Ast::Flags(_) => {}
        // Ast::Literal(_) => {}
        // Ast::Dot(_) => {}
        // Ast::Assertion(_) => {}
        // Ast::ClassUnicode(_) => {}
        // Ast::ClassPerl(_) => {}
        Ast::ClassBracketed(ref class_bracketed) => {
            let ClassBracketed { kind, .. } = *class_bracketed.clone();
            _visit_class_set(kind, groups);
        }
        Ast::Repetition(ref repetition) => {
            let Repetition { ast, .. } = *repetition.clone();
            _visit_node(*ast, groups);
        }
        Ast::Group(ref group) => {
            let Group { kind, ast, .. } = *group.clone();
            match kind {
                GroupKind::CaptureIndex(index) => groups.push((index, None)),
                GroupKind::CaptureName {
                    name: CaptureName { name, index, .. },
                    ..
                } => groups.push((index, Some(name))),
                // GroupKind::NonCapturing(_) => {}
                _ => {}
            }
            _visit_node(*ast, groups);
        }
        Ast::Alternation(ref alternation) => {
            let Alternation { asts, .. } = *alternation.clone();
            for ast in asts {
                _visit_node(ast, groups);
            }
        }
        Ast::Concat(ref concat) => {
            let Concat { asts, .. } = *concat.clone();
            for ast in asts {
                _visit_node(ast, groups);
            }
        }
        _ => {}
    }
}

pub fn get_groups_from_ast(node: Ast) -> Vec<SimplifiedGroup> {
    let mut vec = vec![];
    _visit_node(node, &mut vec);
    vec
}

pub fn get_groups(regex: impl ToString) -> Result<Vec<SimplifiedGroup>> {
    let regex_ast = parse::Parser::new().parse(regex.to_string().as_str())?;
    Ok(get_groups_from_ast(regex_ast))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_groups_with_p() {
        assert_eq!(
            get_groups(r"(?P<name>\w+)@example\.com").unwrap(),
            vec![(1, Some("name".to_string()))]
        );
    }

    #[test]
    fn test_get_groups_none() {
        assert_eq!(get_groups(r"a+b*c?").unwrap(), vec![]);
    }

    #[test]
    fn test_get_groups_numbered() {
        assert_eq!(get_groups(r"(a{2,4})").unwrap(), vec![(1, None)]);
    }

    #[test]
    fn test_get_groups_multiple() {
        assert_eq!(
            get_groups(r"(a{2,4})(?<name>\w+)").unwrap(),
            vec![(1, None), (2, Some("name".to_string()))]
        );
    }
}

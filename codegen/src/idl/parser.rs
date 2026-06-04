//! Parser for the Fluorite IDL using chumsky

use chumsky::prelude::*;
use logos::Logos;

use crate::idl::ast::{
    AstAttribute, AstEnum, AstEnumVariant, AstField, AstFile, AstItem, AstStruct, AstType,
    AstTypeAlias, AstUnion, AstUnionVariant, AstUse, Spanned,
};
use crate::idl::lexer::Token;

/// Parse error type (rendered to an owned string so it can outlive the token buffer)
pub type ParseError = String;

/// Shorthand for the parser bound shared by every combinator in this module.
type Extra<'a> = extra::Err<Rich<'a, Token>>;

/// Convert chumsky's span into the AST's `Range<usize>` span type.
fn to_range(span: SimpleSpan) -> std::ops::Range<usize> {
    span.into_range()
}

/// Parse a complete .fl file from source string
pub fn parse_file(source: &str) -> Result<AstFile, Vec<ParseError>> {
    let tokens = tokenize(source);
    let parser = file_parser();
    parser
        .parse(tokens.as_slice())
        .into_result()
        .map_err(|errors| errors.into_iter().map(|e| e.to_string()).collect())
}

/// Tokenize source string into tokens (spans handled by chumsky)
fn tokenize(source: &str) -> Vec<Token> {
    Token::lexer(source)
        .filter_map(|result| result.ok())
        .collect()
}

/// Parser for a complete file
fn file_parser<'a>() -> impl Parser<'a, &'a [Token], AstFile, Extra<'a>> + Clone {
    // Skip any leading doc comments (file-level documentation)
    doc_comment()
        .repeated()
        .collect::<Vec<_>>()
        .ignore_then(package_stmt())
        .then(use_stmt().repeated().collect::<Vec<_>>())
        .then(item().repeated().collect::<Vec<_>>())
        .map(|((package, uses), items)| AstFile {
            package,
            uses,
            items,
        })
        .then_ignore(end())
}

/// Parser for dotted path: `foo.bar.baz`
fn dotted_path<'a>() -> impl Parser<'a, &'a [Token], Vec<Spanned<String>>, Extra<'a>> + Clone {
    ident()
        .separated_by(just(Token::Dot))
        .at_least(1)
        .collect::<Vec<_>>()
}

/// Parser for package statement: `package com.example.users;`
fn package_stmt<'a>() -> impl Parser<'a, &'a [Token], Vec<Spanned<String>>, Extra<'a>> + Clone {
    just(Token::Package)
        .ignore_then(dotted_path())
        .then_ignore(just(Token::Semi))
}

/// Parser for use statement: `use com.example.users.User;`
fn use_stmt<'a>() -> impl Parser<'a, &'a [Token], AstUse, Extra<'a>> + Clone {
    just(Token::Use)
        .ignore_then(dotted_path())
        .then_ignore(just(Token::Semi))
        .map_with(|path, e| AstUse {
            path,
            span: to_range(e.span()),
        })
}

/// Parser for any top-level item
fn item<'a>() -> impl Parser<'a, &'a [Token], AstItem, Extra<'a>> + Clone {
    choice((
        struct_def().map(AstItem::Struct),
        enum_def().map(AstItem::Enum),
        union_def().map(AstItem::Union),
        type_alias().map(AstItem::TypeAlias),
    ))
}

/// Parser for struct definition
fn struct_def<'a>() -> impl Parser<'a, &'a [Token], AstStruct, Extra<'a>> + Clone {
    doc_comment()
        .repeated()
        .collect::<Vec<_>>()
        .map(|docs| docs.into_iter().next())
        .then(attributes())
        .then_ignore(just(Token::Struct))
        .then(ident())
        .then(struct_body())
        .map_with(|(((doc, attrs), name), fields), e| AstStruct {
            name,
            attrs,
            fields,
            doc,
            span: to_range(e.span()),
        })
}

/// Parser for struct body: `{ fields }`
fn struct_body<'a>() -> impl Parser<'a, &'a [Token], Vec<AstField>, Extra<'a>> + Clone {
    just(Token::LBrace)
        .ignore_then(
            field()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::RBrace))
}

/// Parser for a field
fn field<'a>() -> impl Parser<'a, &'a [Token], AstField, Extra<'a>> + Clone {
    doc_comment()
        .repeated()
        .collect::<Vec<_>>()
        .map(|docs| docs.into_iter().next())
        .then(attributes())
        .then(ident())
        .then_ignore(just(Token::Colon))
        .then(ty())
        .map_with(|(((doc, attrs), name), ty), e| AstField {
            name,
            ty,
            attrs,
            doc,
            span: to_range(e.span()),
        })
}

/// Parser for primitive type tokens (String, bool, i32, etc.)
fn primitive_type<'a>() -> impl Parser<'a, &'a [Token], AstType, Extra<'a>> + Clone {
    let primitives = choice((
        just(Token::TyString).to("String".to_string()),
        just(Token::TyBool).to("bool".to_string()),
        just(Token::TyI32).to("i32".to_string()),
        just(Token::TyI64).to("i64".to_string()),
        just(Token::TyU32).to("u32".to_string()),
        just(Token::TyU64).to("u64".to_string()),
        just(Token::TyF32).to("f32".to_string()),
        just(Token::TyF64).to("f64".to_string()),
        just(Token::TyAny).to("Any".to_string()),
        // Extended types
        just(Token::TyUuid).to("Uuid".to_string()),
        just(Token::TyDecimal).to("Decimal".to_string()),
        just(Token::TyBytes).to("Bytes".to_string()),
        just(Token::TyUrl).to("Url".to_string()),
        just(Token::TyDateTime).to("DateTime".to_string()),
        just(Token::TyDateTimeUtc).to("DateTimeUtc".to_string()),
        just(Token::TyDateTimeTz).to("DateTimeTz".to_string()),
        just(Token::TyDate).to("Date".to_string()),
        just(Token::TyTime).to("Time".to_string()),
        just(Token::TyDuration).to("Duration".to_string()),
        just(Token::TyTimestamp).to("Timestamp".to_string()),
        just(Token::TyTimestampMillis).to("TimestampMillis".to_string()),
    ));

    primitives.map_with(|name, e| AstType::Named(Spanned::new(name, to_range(e.span()))))
}

/// Parser for type expression
fn ty<'a>() -> impl Parser<'a, &'a [Token], AstType, Extra<'a>> + Clone {
    recursive(|ty| {
        // Generic types must be tried first since they start with specific keywords
        let option = just(Token::TyOption)
            .ignore_then(just(Token::LAngle))
            .ignore_then(ty.clone())
            .then_ignore(just(Token::RAngle))
            .map(|inner| AstType::Option(Box::new(inner)));

        let vec = just(Token::TyVec)
            .ignore_then(just(Token::LAngle))
            .ignore_then(ty.clone())
            .then_ignore(just(Token::RAngle))
            .map(|inner| AstType::Vec(Box::new(inner)));

        let map = just(Token::TyMap)
            .ignore_then(just(Token::LAngle))
            .ignore_then(ty.clone())
            .then_ignore(just(Token::Comma))
            .then(ty.clone())
            .then_ignore(just(Token::RAngle))
            .map(|(key, value)| AstType::Map(Box::new(key), Box::new(value)));

        // Primitive types (String, bool, i32, Uuid, etc.)
        let primitive = primitive_type();

        // Custom named types (User, Order, etc.)
        let custom = ident().map(AstType::Named);

        choice((option, vec, map, primitive, custom))
    })
}

/// Parser for enum definition
fn enum_def<'a>() -> impl Parser<'a, &'a [Token], AstEnum, Extra<'a>> + Clone {
    doc_comment()
        .repeated()
        .collect::<Vec<_>>()
        .map(|docs| docs.into_iter().next())
        .then(attributes())
        .then_ignore(just(Token::Enum))
        .then(ident())
        .then(enum_body())
        .map_with(|(((doc, attrs), name), variants), e| AstEnum {
            name,
            attrs,
            variants,
            doc,
            span: to_range(e.span()),
        })
}

/// Parser for enum body: `{ variants }`
fn enum_body<'a>() -> impl Parser<'a, &'a [Token], Vec<AstEnumVariant>, Extra<'a>> + Clone {
    just(Token::LBrace)
        .ignore_then(
            enum_variant()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::RBrace))
}

/// Parser for enum variant
fn enum_variant<'a>() -> impl Parser<'a, &'a [Token], AstEnumVariant, Extra<'a>> + Clone {
    doc_comment()
        .repeated()
        .collect::<Vec<_>>()
        .map(|docs| docs.into_iter().next())
        .then(attributes())
        .then(ident())
        .map_with(|((doc, attrs), name), e| AstEnumVariant {
            name,
            attrs,
            doc,
            span: to_range(e.span()),
        })
}

/// Parser for union definition
fn union_def<'a>() -> impl Parser<'a, &'a [Token], AstUnion, Extra<'a>> + Clone {
    doc_comment()
        .repeated()
        .collect::<Vec<_>>()
        .map(|docs| docs.into_iter().next())
        .then(attributes())
        .then_ignore(just(Token::Union))
        .then(ident())
        .then(union_body())
        .map_with(|(((doc, attrs), name), variants), e| AstUnion {
            name,
            attrs,
            variants,
            doc,
            span: to_range(e.span()),
        })
}

/// Parser for union body: `{ variants }`
fn union_body<'a>() -> impl Parser<'a, &'a [Token], Vec<AstUnionVariant>, Extra<'a>> + Clone {
    just(Token::LBrace)
        .ignore_then(
            union_variant()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::RBrace))
}

/// Parser for union variant: `Variant` or `Variant(Type)`
fn union_variant<'a>() -> impl Parser<'a, &'a [Token], AstUnionVariant, Extra<'a>> + Clone {
    ident()
        .then(
            just(Token::LParen)
                .ignore_then(ident())
                .then_ignore(just(Token::RParen))
                .or_not(),
        )
        .map_with(|(name, inner_type), e| AstUnionVariant {
            name,
            inner_type,
            span: to_range(e.span()),
        })
}

/// Parser for type alias: `type Name = Target;`
fn type_alias<'a>() -> impl Parser<'a, &'a [Token], AstTypeAlias, Extra<'a>> + Clone {
    doc_comment()
        .repeated()
        .collect::<Vec<_>>()
        .map(|docs| docs.into_iter().next())
        .then_ignore(just(Token::Type))
        .then(ident())
        .then_ignore(just(Token::Eq))
        .then(ty())
        .then_ignore(just(Token::Semi))
        .map_with(|((doc, name), target), e| AstTypeAlias {
            name,
            target,
            doc,
            span: to_range(e.span()),
        })
}

/// Parser for attributes: `#[attr]` or `#[attr = "value"]`
fn attributes<'a>() -> impl Parser<'a, &'a [Token], Vec<AstAttribute>, Extra<'a>> + Clone {
    attribute().repeated().collect::<Vec<_>>()
}

fn attribute<'a>() -> impl Parser<'a, &'a [Token], AstAttribute, Extra<'a>> + Clone {
    just(Token::Hash)
        .ignore_then(just(Token::LBracket))
        .ignore_then(ident())
        .then(just(Token::Eq).ignore_then(string_lit()).or_not())
        .then_ignore(just(Token::RBracket))
        .map_with(|(name, value), e| AstAttribute {
            name,
            value,
            span: to_range(e.span()),
        })
}

/// Parser for doc comment as string
fn doc_comment<'a>() -> impl Parser<'a, &'a [Token], String, Extra<'a>> + Clone {
    select! {
        Token::DocComment(s) => s,
    }
}

/// Parser for identifier (including type keywords when used as names)
fn ident<'a>() -> impl Parser<'a, &'a [Token], Spanned<String>, Extra<'a>> + Clone {
    // Accept both Ident tokens and type keywords (which can be used as field names)
    let ident_token = select! {
        Token::Ident(s) => s,
    };

    let type_as_ident = choice((
        just(Token::TyString).to("String".to_string()),
        just(Token::TyBool).to("bool".to_string()),
        just(Token::TyI32).to("i32".to_string()),
        just(Token::TyI64).to("i64".to_string()),
        just(Token::TyU32).to("u32".to_string()),
        just(Token::TyU64).to("u64".to_string()),
        just(Token::TyF32).to("f32".to_string()),
        just(Token::TyF64).to("f64".to_string()),
        just(Token::TyOption).to("Option".to_string()),
        just(Token::TyVec).to("Vec".to_string()),
        just(Token::TyMap).to("Map".to_string()),
        just(Token::TyAny).to("Any".to_string()),
        just(Token::TyUuid).to("Uuid".to_string()),
        just(Token::TyDecimal).to("Decimal".to_string()),
        just(Token::TyBytes).to("Bytes".to_string()),
        just(Token::TyUrl).to("Url".to_string()),
        just(Token::TyDateTime).to("DateTime".to_string()),
        just(Token::TyDateTimeUtc).to("DateTimeUtc".to_string()),
        just(Token::TyDateTimeTz).to("DateTimeTz".to_string()),
        just(Token::TyDate).to("Date".to_string()),
        just(Token::TyTime).to("Time".to_string()),
        just(Token::TyDuration).to("Duration".to_string()),
        just(Token::TyTimestamp).to("Timestamp".to_string()),
        just(Token::TyTimestampMillis).to("TimestampMillis".to_string()),
    ));

    ident_token
        .or(type_as_ident)
        .map_with(|s, e| Spanned::new(s, to_range(e.span())))
}

/// Parser for string literal
fn string_lit<'a>() -> impl Parser<'a, &'a [Token], Spanned<String>, Extra<'a>> + Clone {
    select! {
        Token::StringLit(s) => s,
    }
    .map_with(|s, e| Spanned::new(s, to_range(e.span())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_package() {
        let source = "package orders;";
        let result = parse_file(source);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.package.len(), 1);
        assert_eq!(ast.package[0].value, "orders");
    }

    #[test]
    fn test_parse_dotted_package() {
        let source = "package com.example.users;";
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.package.len(), 3);
        assert_eq!(ast.package[0].value, "com");
        assert_eq!(ast.package[1].value, "example");
        assert_eq!(ast.package[2].value, "users");
    }

    #[test]
    fn test_parse_deep_dotted_path() {
        let source = "package a.b.c.d.e.f;";
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.package.len(), 6);
        assert_eq!(ast.package[0].value, "a");
        assert_eq!(ast.package[5].value, "f");
    }

    #[test]
    fn test_parse_use() {
        let source = r#"
            package test;
            use com.example.users.User;
        "#;
        let result = parse_file(source);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.uses.len(), 1);
        assert_eq!(ast.uses[0].path.len(), 4);
        assert_eq!(ast.uses[0].path[0].value, "com");
        assert_eq!(ast.uses[0].path[1].value, "example");
        assert_eq!(ast.uses[0].path[2].value, "users");
        assert_eq!(ast.uses[0].path[3].value, "User");
    }

    #[test]
    fn test_parse_dotted_use() {
        let source = r#"
            package test;
            use com.example.users.User;
            use com.example.orders.Order;
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.uses.len(), 2);
        assert_eq!(ast.uses[0].path.len(), 4);
        assert_eq!(ast.uses[1].path.len(), 4);
    }

    #[test]
    fn test_parse_struct() {
        let source = r#"
            package test;
            struct User {
                name: String,
                age: u32,
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);
        match &ast.items[0] {
            AstItem::Struct(s) => {
                assert_eq!(s.name.value, "User");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name.value, "name");
                assert_eq!(s.fields[1].name.value, "age");
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_enum() {
        let source = r#"
            package test;
            enum Status {
                Active,
                Inactive,
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);
        match &ast.items[0] {
            AstItem::Enum(e) => {
                assert_eq!(e.name.value, "Status");
                assert_eq!(e.variants.len(), 2);
                assert_eq!(e.variants[0].name.value, "Active");
                assert_eq!(e.variants[1].name.value, "Inactive");
            }
            _ => panic!("Expected enum"),
        }
    }

    #[test]
    fn test_parse_union() {
        let source = r#"
            package test;
            union Event {
                UserCreated(User),
                OrderPlaced(Order),
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);
        match &ast.items[0] {
            AstItem::Union(u) => {
                assert_eq!(u.name.value, "Event");
                assert_eq!(u.variants.len(), 2);
                assert_eq!(u.variants[0].name.value, "UserCreated");
                assert!(u.variants[0].inner_type.is_some());
                assert_eq!(u.variants[0].inner_type.as_ref().unwrap().value, "User");
            }
            _ => panic!("Expected union"),
        }
    }

    #[test]
    fn test_parse_type_alias() {
        let source = r#"
            package test;
            type OrderList = Vec<Order>;
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);
        match &ast.items[0] {
            AstItem::TypeAlias(t) => {
                assert_eq!(t.name.value, "OrderList");
            }
            _ => panic!("Expected type alias"),
        }
    }

    #[test]
    fn test_parse_with_doc_comment() {
        let source = r#"
            package test;
            /// A user in the system
            struct User {
                /// The user's name
                name: String,
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        match &ast.items[0] {
            AstItem::Struct(s) => {
                assert_eq!(s.doc.as_ref().unwrap(), "A user in the system");
                assert_eq!(s.fields[0].doc.as_ref().unwrap(), "The user's name");
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_with_attributes() {
        let source = r#"
            package test;
            #[rename = "user_name"]
            struct User {
                #[deprecated]
                name: String,
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        match &ast.items[0] {
            AstItem::Struct(s) => {
                assert_eq!(s.attrs.len(), 1);
                assert_eq!(s.attrs[0].name.value, "rename");
                assert_eq!(s.attrs[0].value.as_ref().unwrap().value, "user_name");
                assert_eq!(s.fields[0].attrs.len(), 1);
                assert_eq!(s.fields[0].attrs[0].name.value, "deprecated");
                assert!(s.fields[0].attrs[0].value.is_none());
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_complex_types() {
        let source = r#"
            package test;
            struct Data {
                items: Vec<String>,
                maybe: Option<i32>,
                mapping: Map<String, User>,
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
    }
}

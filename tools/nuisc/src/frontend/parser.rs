use nuis_semantics::model::{
    AstAttribute, AstAttributeArg, AstAttributeValue, AstBinaryOp, AstConstItem, AstEnumDef,
    AstEnumVariant, AstEnumVariantKind, AstExpr, AstExternFunction, AstExternInterface,
    AstFunction, AstGenericParam, AstImplDef, AstImplMethod, AstMatchArm, AstMatchPattern,
    AstModule, AstParam, AstStmt, AstStructDef, AstStructField, AstTraitDef, AstTraitMethodSig,
    AstTypeAlias, AstTypeRef, AstVisibility, AstWherePredicate, TestClockDomain, TestClockPolicy,
};

use super::lexer::{describe_token, Token};

#[path = "parser_attributes.rs"]
mod parser_attributes;
#[path = "parser_blocks.rs"]
mod parser_blocks;
#[path = "parser_destructure.rs"]
mod parser_destructure;
#[path = "parser_exprs.rs"]
mod parser_exprs;
#[path = "parser_externs.rs"]
mod parser_externs;
#[path = "parser_items.rs"]
mod parser_items;
#[path = "parser_match_patterns.rs"]
mod parser_match_patterns;
#[path = "parser_statements.rs"]
mod parser_statements;
#[path = "parser_test_metadata.rs"]
mod parser_test_metadata;
#[path = "parser_tokens.rs"]
mod parser_tokens;
#[path = "parser_types.rs"]
mod parser_types;

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    allow_struct_literals: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            cursor: 0,
            allow_struct_literals: true,
        }
    }

    pub fn parse_module(&mut self) -> Result<AstModule, String> {
        let mut attributes = self.consume_module_leading_doc_comments()?;
        let mut uses = Vec::new();
        let mut externs = Vec::new();
        let mut extern_interfaces = Vec::new();
        while self.peek_word("use") {
            uses.push(self.parse_use_decl()?);
        }
        attributes.extend(self.consume_module_leading_doc_comments()?);
        while self.peek_item_keyword_after_attributes("extern") {
            let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
            if !attributes.is_empty() {
                return Err(
                    "module-level extern declarations currently only support `pub` before `extern`"
                        .to_owned(),
                );
            }
            let abi = self.parse_extern_abi()?;
            if self.peek_word("interface") {
                extern_interfaces.push(self.parse_extern_interface(visibility, abi)?);
            } else {
                externs.push(self.parse_extern_function_with_abi(visibility, abi, None)?);
            }
        }
        attributes.extend(self.consume_module_leading_doc_comments()?);
        self.expect_word("mod")?;
        let domain = self.expect_ident()?;
        let unit = self.expect_ident()?;
        self.expect_symbol('{')?;

        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        let mut consts = Vec::new();
        let mut type_aliases = Vec::new();
        let mut functions = Vec::new();
        while !self.peek_symbol('}') {
            if self.peek_word("mod") {
                return Err("nested mod definitions are not allowed".to_owned());
            }
            if self.peek_item_keyword_after_attributes("extern") {
                let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
                if !attributes.is_empty() {
                    return Err(
                        "module-level extern declarations currently only support `pub` before `extern`"
                            .to_owned(),
                    );
                }
                let abi = self.parse_extern_abi()?;
                if self.peek_word("interface") {
                    extern_interfaces.push(self.parse_extern_interface(visibility, abi)?);
                } else {
                    externs.push(self.parse_extern_function_with_abi(visibility, abi, None)?);
                }
            } else if self.peek_item_keyword_after_attributes("struct") {
                structs.push(self.parse_struct_def()?);
            } else if self.peek_item_keyword_after_attributes("enum") {
                enums.push(self.parse_enum_def()?);
            } else if self.peek_item_keyword_after_attributes("trait") {
                traits.push(self.parse_trait_def()?);
            } else if self.peek_item_keyword_after_attributes("impl") {
                impls.push(self.parse_impl_def()?);
            } else if self.peek_item_keyword_after_attributes("const") {
                consts.push(self.parse_const_item()?);
            } else if self.peek_item_keyword_after_attributes("type") {
                type_aliases.push(self.parse_type_alias_item()?);
            } else {
                functions.push(self.parse_function()?);
            }
        }

        self.expect_symbol('}')?;
        self.expect_eof()?;

        Ok(AstModule {
            attributes,
            uses,
            domain,
            unit,
            externs,
            extern_interfaces,
            consts,
            type_aliases,
            structs,
            enums,
            traits,
            impls,
            functions,
        })
    }

    fn parse_use_decl(&mut self) -> Result<nuis_semantics::model::AstUse, String> {
        self.expect_word("use")?;
        let domain = self.expect_ident()?;
        let unit = self.expect_ident()?;
        self.expect_symbol(';')?;
        Ok(nuis_semantics::model::AstUse { domain, unit })
    }
}

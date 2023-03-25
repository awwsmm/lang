use std::iter::Peekable;

use rowan::{Checkpoint, GreenNode, GreenNodeBuilder, Language};

use crate::expr::expr;
use crate::lexer::{Lexer, SyntaxKind};
use crate::syntax::{LangLanguage, SyntaxNode};

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    builder: GreenNodeBuilder<'static>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input).peekable(),
            builder: GreenNodeBuilder::new(),
        }
    }

    pub fn parse(mut self) -> Parse {
        self.start_node(SyntaxKind::Root);

        expr(&mut self);

        self.finish_node();

        Parse {
            green_node: self.builder.finish(),
        }
    }

    pub(crate) fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder
            .start_node_at(checkpoint, LangLanguage::kind_to_raw(kind));
    }

    fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(LangLanguage::kind_to_raw(kind))
    }

    pub(crate) fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    pub(crate) fn checkpoint(&self) -> Checkpoint {
        self.builder.checkpoint()
    }

    pub(crate) fn peek(&mut self) -> Option<SyntaxKind> {
        self.lexer.peek().map(|(kind, _)| *kind)
    }

    pub(crate) fn bump(&mut self) {
        let (kind, text) = self.lexer.next().unwrap();

        self.builder
            .token(LangLanguage::kind_to_raw(kind), text.into());
    }
}

pub struct Parse {
    green_node: GreenNode,
}

impl Parse {
    pub fn debug_tree(&self) -> String {
        let syntax_node  = SyntaxNode::new_root(self.green_node.clone());
        let formatted = format!("{:#?}", syntax_node);

        // We cut off the last byte because formatting the SyntaxNode adds on a newline at the end.
        formatted[0..formatted.len() - 1].to_string()
    }
}

#[cfg(test)]
pub(crate) fn check(input: &str, expected_tree: expect_test::Expect) {
    let parse = Parser::new(input).parse();
    expected_tree.assert_eq(&parse.debug_tree());
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn parse_nothing() {
        check("", expect![[r#"Root@0..0"#]]);
    }

    #[test]
    fn parse_simple_binary_expression() {
        check(
            "1+2",
            expect![[r#"
Root@0..3
  BinaryExpr@0..3
    Number@0..1 "1"
    Plus@1..2 "+"
    Number@2..3 "2""#]],
        );
    }

    #[test]
    fn parse_left_associative_binary_expression() {
        check(
            "1+2+3+4",
            expect![[r#"
Root@0..7
  BinaryExpr@0..7
    BinaryExpr@0..5
      BinaryExpr@0..3
        Number@0..1 "1"
        Plus@1..2 "+"
        Number@2..3 "2"
      Plus@3..4 "+"
      Number@4..5 "3"
    Plus@5..6 "+"
    Number@6..7 "4""#]],
        );
    }

    #[test]
    fn parse_binary_expression_with_mixed_binding_power() {
        check(
            "1+2*3-4",
            expect![[r#"
Root@0..7
  BinaryExpr@0..7
    BinaryExpr@0..5
      Number@0..1 "1"
      Plus@1..2 "+"
      BinaryExpr@2..5
        Number@2..3 "2"
        Star@3..4 "*"
        Number@4..5 "3"
    Minus@5..6 "-"
    Number@6..7 "4""#]],
        );
    }

    #[test]
    fn parse_negation() {
        check(
            "-10",
            expect![[r#"
Root@0..3
  PrefixExpr@0..3
    Minus@0..1 "-"
    Number@1..3 "10""#]],
        );
    }

    #[test]
    fn negation_has_higher_binding_power_than_infix_operators() {
        check(
            "-20+20",
            expect![[r#"
Root@0..6
  BinaryExpr@0..6
    PrefixExpr@0..3
      Minus@0..1 "-"
      Number@1..3 "20"
    Plus@3..4 "+"
    Number@4..6 "20""#]],
        );
    }
}
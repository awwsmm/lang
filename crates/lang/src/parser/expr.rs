use crate::lexer::SyntaxKind;
use crate::parser::Parser;

pub(crate) fn expr(p: &mut Parser) {
    expr_binding_power(p, 0);
}


fn expr_binding_power(p: &mut Parser, minimum_binding_power: u8) {
    let checkpoint = p.checkpoint();

    match p.peek() {
        Some(SyntaxKind::Number) | Some(SyntaxKind::Ident) => p.bump(),
        Some(SyntaxKind::Minus) => {
            let op = PrefixOp::Neg;
            let ((), right_binding_power) = op.binding_power();

            // Eat the operator’s token.
            p.bump();

            p.start_node_at(checkpoint, SyntaxKind::PrefixExpr);
            expr_binding_power(p, right_binding_power);
            p.finish_node();
        }
        Some(SyntaxKind::LParen) => {
            p.bump();
            expr_binding_power(p, 0);

            assert_eq!(p.peek(), Some(SyntaxKind::RParen));
            p.bump();
        }
        _ => {}
    }

    loop {
        let op = match p.peek() {
            Some(SyntaxKind::Plus) => InfixOp::Add,
            Some(SyntaxKind::Minus) => InfixOp::Sub,
            Some(SyntaxKind::Star) => InfixOp::Mul,
            Some(SyntaxKind::Slash) => InfixOp::Div,
            _ => return, // we'll handle errors later.
        };

        let (left_binding_power, right_binding_power) = op.binding_power();

        if left_binding_power < minimum_binding_power {
            return;
        }

        // Eat the operator's token
        p.bump();

        p.start_node_at(checkpoint, SyntaxKind::BinaryExpr);
        expr_binding_power(p, right_binding_power);
        p.finish_node();
    }
}

enum InfixOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl InfixOp {
    fn binding_power(&self) -> (u8, u8) {
        match self {
            Self::Add | Self::Sub => (1, 2),
            Self::Mul | Self::Div => (3, 4),
        }
    }
}

enum PrefixOp {
    Neg,
}

impl PrefixOp {
    fn binding_power(&self) -> ((), u8) {
        match self {
            Self::Neg => ((), 5),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::parser::check;

    #[test]
    fn parse_number() {
        check(
            "123",
            expect![[r#"
Root@0..3
  Number@0..3 "123""#]],
        );
    }

    #[test]
    fn parse_variable_ref() {
        check(
            "counter",
            expect![[r#"
Root@0..7
  Ident@0..7 "counter""#]],
        );
    }

    #[test]
    fn parse_nested_parentheses() {
        check(
            "((((((10))))))",
            expect![[r#"
Root@0..14
  LParen@0..1 "("
  LParen@1..2 "("
  LParen@2..3 "("
  LParen@3..4 "("
  LParen@4..5 "("
  LParen@5..6 "("
  Number@6..8 "10"
  RParen@8..9 ")"
  RParen@9..10 ")"
  RParen@10..11 ")"
  RParen@11..12 ")"
  RParen@12..13 ")"
  RParen@13..14 ")""#]],
        );
    }

    #[test]
    fn parentheses_affect_precedence() {
        check(
            "5*(2+1)",
            expect![[r#"
Root@0..7
  BinaryExpr@0..7
    Number@0..1 "5"
    Star@1..2 "*"
    LParen@2..3 "("
    BinaryExpr@3..6
      Number@3..4 "2"
      Plus@4..5 "+"
      Number@5..6 "1"
    RParen@6..7 ")""#]],
        );
    }
}
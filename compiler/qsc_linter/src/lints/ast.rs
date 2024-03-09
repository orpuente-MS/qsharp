// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::push_lint;
use crate::linter::{
    ast::{declare_ast_lints, AstLintPass},
    Lint, LintLevel,
};
use qsc_ast::ast::{BinOp, ExprKind, Lit, StmtKind};
use qsc_data_structures::span::Span;

declare_ast_lints! {
    (DivisionByZero, LintLevel::Allow, "attempt to divide by zero"),
    (NeedlessParens, LintLevel::Warning, "unnecessary parentheses"),
    (RedundantSemicolons, LintLevel::Warning, "redundant semicolons"),
}

impl AstLintPass for DivisionByZero {
    fn check_expr(expr: &qsc_ast::ast::Expr, buffer: &mut Vec<Lint>) {
        if let ExprKind::BinOp(BinOp::Div, _, ref rhs) = *expr.kind {
            if let ExprKind::Lit(ref lit) = *rhs.kind {
                if let Lit::Int(0) = **lit {
                    push_lint!(Self, expr.span, buffer);
                }
            }
        }
    }
}

impl AstLintPass for NeedlessParens {
    /// The idea is that if we find a expr of the form:
    /// a + (expr)
    /// and `expr` has higher precedence than `+`, then the
    /// parentheses are needless. Parentheses around a literal
    /// are also needless.
    fn check_expr(expr: &qsc_ast::ast::Expr, buffer: &mut Vec<Lint>) {
        fn push(parent: &qsc_ast::ast::Expr, child: &qsc_ast::ast::Expr, buf: &mut Vec<Lint>) {
            if let ExprKind::Paren(expr) = &*child.kind {
                if precedence(parent) > precedence(expr) {
                    push_lint!(NeedlessParens, child.span, buf);
                }
            }
        }

        match &*expr.kind {
            ExprKind::BinOp(_, left, right) => {
                push(expr, left, buffer);
                push(expr, right, buffer);
            }
            ExprKind::Assign(_, right) | ExprKind::AssignOp(_, _, right) => {
                push(expr, right, buffer);
            }
            _ => (),
        }
    }

    fn check_stmt(stmt: &qsc_ast::ast::Stmt, buffer: &mut Vec<Lint>) {
        if let StmtKind::Local(_, _, right) = &*stmt.kind {
            if let ExprKind::Paren(_) = &*right.kind {
                push_lint!(NeedlessParens, right.span, buffer);
            }
        }
    }
}

impl AstLintPass for RedundantSemicolons {
    /// Checks if there are redundant semicolons. The idea is that a redundant
    /// semicolon is parsed as an Empty statement. If we have multiple empty
    /// statements in a row, we group them as single lint, that spans from
    /// the first redundant semicolon to the last redundant semicolon.
    fn check_block(block: &qsc_ast::ast::Block, buffer: &mut Vec<Lint>) {
        /// Helper function that pushes a lint to the buffer if we have
        /// found two or more semicolons.
        fn maybe_push(seq: &mut Option<Span>, buffer: &mut Vec<Lint>) {
            if let Some(span) = seq.take() {
                push_lint!(RedundantSemicolons, span, buffer);
            }
        }

        // a finte state machine that keeps track of the span of the redundant semicolons
        // None: no redundant semicolons
        // Some(_): one or more redundant semicolons
        let mut seq: Option<Span> = None;

        for stmt in block.stmts.iter() {
            match (&*stmt.kind, &mut seq) {
                (StmtKind::Empty, None) => seq = Some(stmt.span),
                (StmtKind::Empty, Some(span)) => span.hi = stmt.span.hi,
                (_, seq) => maybe_push(seq, buffer),
            }
        }

        maybe_push(&mut seq, buffer);
    }
}

fn precedence(expr: &qsc_ast::ast::Expr) -> u8 {
    match &*expr.kind {
        ExprKind::Lit(_) => 0,
        ExprKind::Paren(_) => 1,
        ExprKind::UnOp(_, _) => 2,
        ExprKind::BinOp(op, _, _) => match op {
            BinOp::Exp => 3,
            BinOp::Div | BinOp::Mod | BinOp::Mul => 4,
            BinOp::Add | BinOp::Sub => 5,
            BinOp::Shl | BinOp::Shr => 6,
            BinOp::Gt | BinOp::Gte | BinOp::Lt | BinOp::Lte => 7,
            BinOp::Eq | BinOp::Neq => 8,
            BinOp::OrB | BinOp::XorB | BinOp::AndB => 9,
            BinOp::OrL | BinOp::AndL => 10,
        },
        ExprKind::Assign(_, _) | ExprKind::AssignOp(_, _, _) => 11,
        _ => u8::MAX,
    }
}

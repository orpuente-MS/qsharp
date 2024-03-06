// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(super) mod ast;

macro_rules! declare_lint {
    ($lint_name:ident, $level:expr, $msg:expr) => {
        pub(crate) struct $lint_name;

        impl $lint_name {
            const LEVEL: LintLevel = $level;
            const MESSAGE: &'static str = $msg;
        }
    };
}

macro_rules! push_lint {
    ($lint_ty:ty, $span:expr, $buffer:expr) => {
        $buffer.push(Lint {
            span: $span,
            message: <$lint_ty>::MESSAGE,
            level: <$lint_ty>::LEVEL,
        })
    };
}

pub(crate) use declare_lint;
pub(crate) use push_lint;

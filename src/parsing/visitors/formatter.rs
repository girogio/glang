use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::parsing::ast::{AstNode, Visitor};

pub struct AstFormatter {
    tab_level: usize,
    buff: BufWriter<File>,
}

impl AstFormatter {
    pub fn new(file: &Path) -> Self {
        let file = File::create(file).unwrap();

        Self {
            tab_level: 0,
            buff: BufWriter::new(file),
        }
    }
}

impl Visitor for AstFormatter {
    fn visit(&mut self, node: &AstNode) {
        match node {
            AstNode::Program { statements } => {
                for statement in statements {
                    self.visit(statement);
                    if let AstNode::Assignment { .. } = statement {
                        writeln!(self.buff, ";");
                    }
                    writeln!(self.buff, "\n");
                }
            }

            AstNode::VarDec {
                identifier,
                var_type,
                expression,
            } => {
                write!(self.buff, "let ");
                self.visit(identifier);
                match var_type {
                    Some(var_type) => write!(self.buff, ": {}", var_type.span.lexeme),
                    None => write!(self.buff, ": n/a"),
                };
                write!(self.buff, " = ");
                self.visit(expression);
                write!(self.buff, ";");
            }

            AstNode::Delay { expression } => {
                write!(self.buff, "__delay ");
                self.visit(expression);
                write!(self.buff, ";");
            }

            AstNode::Print { expression } => {
                write!(self.buff, "__print ");
                self.visit(expression);
                write!(self.buff, ";");
            }

            AstNode::If {
                condition,
                if_true,
                if_false,
            } => {
                write!(self.buff, "if (");
                self.visit(condition);
                write!(self.buff, ") ");
                self.visit(if_true);
            }

            AstNode::Assignment {
                identifier,
                expression,
            } => {
                self.visit(identifier);
                write!(self.buff, " = ");
                self.visit(expression);
            }

            AstNode::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                write!(self.buff, "for (");

                // This is to avoid printing newline after the variable declaration in a for loop
                if let Some(AstNode::VarDec {
                    identifier,
                    var_type,
                    expression,
                }) = initializer.as_ref()
                {
                    write!(self.buff, "let ");
                    self.visit(identifier);
                    if let Some(var_type) = var_type {
                        write!(self.buff, ": {} = ", var_type.span.lexeme);
                    }
                    self.visit(expression);
                }

                write!(self.buff, "; ");

                self.visit(condition);

                write!(self.buff, "; ");
                match increment.as_ref() {
                    Some(increment) => {
                        self.visit(increment);
                    }
                    None => {}
                }
                write!(self.buff, ") ");
                self.visit(body);
            }

            AstNode::Return { expression } => {
                write!(self.buff, "return ");
                self.visit(expression);
                write!(self.buff, ";");
            }

            AstNode::Block { statements } => {
                writeln!(self.buff, "\n{}{{", "  ".repeat(self.tab_level));
                self.tab_level += 1;
                for statement in statements {
                    write!(self.buff, "{}", "  ".repeat(self.tab_level));
                    self.visit(statement);
                    if let AstNode::Assignment { .. } = statement {
                        writeln!(self.buff, ";");
                    } else {
                        write!(self.buff, "\n");
                    }
                }
                self.tab_level -= 1;

                write!(self.buff, "{}}}", "  ".repeat(self.tab_level));
            }

            AstNode::Expression {
                casted_type,
                bin_op,
            } => {
                self.visit(bin_op);
                match casted_type {
                    Some(casted_type) => {
                        write!(self.buff, " as {}", casted_type.span.lexeme);
                    }
                    None => {}
                }
            }

            AstNode::FunctionDecl {
                identifier,
                params,
                return_type,
                block,
            } => {
                write!(self.buff, "fun ");
                self.visit(identifier);
                write!(self.buff, "(");

                let (params, last) = params.split_at(params.len() - 1);

                for param in params {
                    self.visit(param);
                    write!(self.buff, ", ");
                }

                if let Some(last) = last.first() {
                    self.visit(last);
                }

                write!(self.buff, ") -> {}", return_type.span.lexeme);
                self.visit(block);
            }

            AstNode::While { condition, body } => {
                write!(self.buff, "while (");
                let prev_tab_level = self.tab_level;
                self.tab_level = 0;
                self.visit(condition);
                self.tab_level = prev_tab_level;
                write!(self.buff, ") ");
                self.visit(body);
            }

            AstNode::PadWrite {
                loc_x,
                loc_y,
                colour,
            } => {
                write!(self.buff, "__write ");
                self.visit(loc_x);
                write!(self.buff, ", ");
                self.visit(loc_y);
                write!(self.buff, ", ");
                self.visit(colour);
                write!(self.buff, ";");
            }

            AstNode::PadClear { expr } => {
                write!(self.buff, "__clear ");

                self.visit(expr);

                write!(self.buff, ";");
            }

            AstNode::PadWriteBox {
                loc_x,
                loc_y,
                width,
                height,
                colour,
            } => {
                write!(self.buff, "__write_box ");
                self.visit(loc_x);
                write!(self.buff, ", ");
                self.visit(loc_y);
                write!(self.buff, ", ");
                self.visit(width);
                write!(self.buff, ", ");
                self.visit(height);
                write!(self.buff, ", ");
                self.visit(colour);
                write!(self.buff, ";");
            }

            AstNode::BinOp {
                left,
                operator,
                right,
            } => {
                write!(self.buff, "(");
                self.visit(left);
                write!(self.buff, ")");
                write!(self.buff, " {} ", operator.span.lexeme);
                write!(self.buff, "(");
                self.visit(right);
                write!(self.buff, ")");
            }

            AstNode::UnaryOp { operator, expr } => {
                write!(self.buff, "{} ", operator.span.lexeme);
                self.visit(expr);
            }

            AstNode::PadRead { first, second } => {
                write!(self.buff, "__read ");
                self.visit(first);
                write!(self.buff, ", ");
                self.visit(second);
            }

            AstNode::FormalParam {
                identifier,
                param_type,
            } => {
                self.visit(identifier);
                write!(self.buff, ": {}", param_type.span.lexeme);
            }

            AstNode::PadRandI { upper_bound } => {
                write!(self.buff, "__randi ");
                self.visit(upper_bound);
            }

            AstNode::FunctionCall { identifier, args } => {
                self.visit(identifier);
                write!(self.buff, "(");

                let (args, last) = args.split_at(args.len() - 1);

                for arg in args {
                    self.visit(arg);
                    write!(self.buff, ", ");
                }

                if let Some(last) = last.first() {
                    self.visit(last);
                }

                write!(self.buff, ")");
            }

            AstNode::Identifier { token } => {
                write!(self.buff, "{}", token.span.lexeme);
            }

            AstNode::IntLiteral(token) => {
                write!(self.buff, "{}", token.span.lexeme);
            }

            AstNode::FloatLiteral(token) => {
                write!(self.buff, "{}", token.span.lexeme);
            }

            AstNode::BoolLiteral(token) => {
                write!(self.buff, "{}", token.span.lexeme);
            }

            AstNode::ColourLiteral(token) => {
                write!(self.buff, "{}", token.span.lexeme);
            }
            AstNode::PadWidth => {
                write!(self.buff, "__width");
            }
            AstNode::PadHeight => {
                write!(self.buff, "__height");
            }
            AstNode::ActualParams { params } => todo!(),
            AstNode::EndOfFile => {}
        }
    }
}

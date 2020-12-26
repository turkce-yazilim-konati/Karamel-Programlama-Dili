use crate::types::*;
use crate::syntax::{SyntaxParser, SyntaxParserTrait, SyntaxFlag};
use crate::syntax::unary::UnaryParser;
use crate::syntax::util::update_functions_for_temp_return;
use crate::compiler::ast::BramaAstType;

pub struct ModuloParser;
pub struct MultiplyDivideParser;
pub struct AddSubtractParser;

impl SyntaxParserTrait for ModuloParser {
    fn parse(parser: &SyntaxParser) -> AstResult {
        return parse_binary::<MultiplyDivideParser>(parser, &[BramaOperatorType::Modulo]);
    }
}

impl SyntaxParserTrait for MultiplyDivideParser {
    fn parse(parser: &SyntaxParser) -> AstResult {
        return parse_binary::<UnaryParser>(parser, &[BramaOperatorType::Multiplication, BramaOperatorType::Division]);
    }
}

impl SyntaxParserTrait for AddSubtractParser {
    fn parse(parser: &SyntaxParser) -> AstResult {
        return parse_binary::<ModuloParser>(parser, &[BramaOperatorType::Addition, BramaOperatorType::Subtraction]);
    }
}

pub fn parse_binary<T: SyntaxParserTrait>(parser: &SyntaxParser, operators: &[BramaOperatorType]) -> AstResult {
    let mut functions_updated_for_temp = false;
    let mut left_expr = T::parse(parser)?;
    match left_expr {
        BramaAstType::None => return Ok(left_expr),
        _ => ()
    };

    loop {
        parser.backup();
        parser.cleanup_whitespaces();
        
        if let Some(operator) = parser.match_operator(operators) {
            if !functions_updated_for_temp {
                update_functions_for_temp_return(&mut left_expr);
                functions_updated_for_temp = true;
            }
            
            parser.cleanup_whitespaces();
            let parser_flags  = parser.flags.get();
            parser.flags.set(parser_flags | SyntaxFlag::IN_EXPRESSION);
            
            let right_expr = T::parse(parser);
            match right_expr {
                Ok(BramaAstType::None) => return Err(("Right side of expression not found", 0, 0)),
                Ok(_) => (),
                Err(_) => return right_expr
            };

            parser.flags.set(parser_flags);
            left_expr = BramaAstType::Binary {
                left: Box::new(left_expr),
                operator: operator,
                right: Box::new(right_expr.unwrap())
            };
        }
        else {
            parser.restore();
            break;
        }
    }

    return Ok(left_expr);
}

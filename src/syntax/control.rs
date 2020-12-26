use crate::types::*;
use crate::syntax::{SyntaxParser, SyntaxParserTrait, SyntaxFlag};
use crate::syntax::binary::AddSubtractParser;
use crate::syntax::util::update_functions_for_temp_return;
use crate::compiler::ast::BramaAstType;

pub struct ExpressionParser;
pub struct OrParser;
pub struct AndParser;
pub struct EqualityParser;
pub struct ControlParser;

impl SyntaxParserTrait for ExpressionParser {
    fn parse(parser: &SyntaxParser) -> AstResult {
        return OrParser::parse(parser);
    }
}

impl SyntaxParserTrait for OrParser {
    fn parse(parser: &SyntaxParser) -> AstResult {
        return parse_control::<AndParser>(parser, &[BramaOperatorType::Or]);
    }
}

impl SyntaxParserTrait for AndParser {
    fn parse(parser: &SyntaxParser) -> AstResult {
        return parse_control::<EqualityParser>(parser, &[BramaOperatorType::And]);
    }
}

impl SyntaxParserTrait for EqualityParser {
    fn parse(parser: &SyntaxParser) -> AstResult {
        return parse_control::<ControlParser>(parser, &[BramaOperatorType::Equal, BramaOperatorType::NotEqual]);
    }
}

impl SyntaxParserTrait for ControlParser {
    fn parse(parser: &SyntaxParser) -> AstResult {
        return parse_control::<AddSubtractParser>(parser, &[BramaOperatorType::GreaterEqualThan, 
            BramaOperatorType::GreaterThan,
            BramaOperatorType::LessEqualThan, 
            BramaOperatorType::LessThan]);
    }
}

pub fn parse_control<T: SyntaxParserTrait>(parser: &SyntaxParser, operators: &[BramaOperatorType]) -> AstResult {
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
            left_expr = BramaAstType::Control {
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

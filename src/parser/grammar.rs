use rust_decimal::Decimal;
use smol_str::SmolStr;
use std::str::FromStr;

use crate::ast::{
  expr::{Expr, Op},
  stmt::Stmt,
  value::Value,
};

peg::parser!(
pub grammar parser() for str {
  pub rule program() -> Vec<Stmt>
    = s:statement()* { s }

  /// Parse program with source location info for each statement
  pub rule program_with_spans() -> Vec<(usize, Stmt)>
    = s:statement_with_pos()* { s }

  /// Statement with position info (byte offset)
  rule statement_with_pos() -> (usize, Stmt)
    = whitespace()?
    pos:position!()
    s:(
        assignment()
      / if_stmt()
      / expr_stmt()
    )
    whitespace()? { (pos, s) }

  pub rule statement() -> Stmt
    = whitespace()?
    s:(
        assignment()
      / if_stmt()
      / expr_stmt()
    )
    whitespace()? { s }

  pub rule expression() -> Expr
    = binary_op()

  pub rule mul_div() -> Expr =
    left:power() mul_div_right:(
        _ op:$("*" / "/" / "%") _ right:power()
        { (op, right) }
    )* {
        let mut result = left;
        for (op, right) in mul_div_right {
            result = match op {
                "*" => Expr::BinaryOp(Box::new(result), Op::Mul, Box::new(right)),
                "/" => Expr::BinaryOp(Box::new(result), Op::Div, Box::new(right)),
                "%" => Expr::BinaryOp(Box::new(result), Op::Mod, Box::new(right)),
                _ => unreachable!()
            };
        }
        result
    }

  pub rule power() -> Expr =
    base:postfix() _ "**" _ exp:power() { Expr::BinaryOp(Box::new(base), Op::Pow, Box::new(exp)) }
    / a:postfix() { a }


  pub rule binary_op() -> Expr = precedence!{
    i:identifier() _ "(" args:((_ e:expression() _ {e}) ** ",") ")" { Expr::FunctionCall(i, args) }
    --
    x:@ _ "&&" _ y:(@) { Expr::BinaryOp(Box::new(x), Op::And, Box::new(y)) }
    x:@ _ "||" _ y:(@) { Expr::BinaryOp(Box::new(x), Op::Or, Box::new(y)) }
    --
    x:@ _ "==" _ y:(@) { Expr::BinaryOp(Box::new(x), Op::Eq, Box::new(y)) }
    x:@ _ "!=" _ y:(@) { Expr::BinaryOp(Box::new(x), Op::Neq, Box::new(y)) }
    x:@ _ "<"  _ y:(@) { Expr::BinaryOp(Box::new(x), Op::Lt, Box::new(y)) }
    x:@ _ "<=" _ y:(@) { Expr::BinaryOp(Box::new(x), Op::Lte, Box::new(y)) }
    x:@ _ ">"  _ y:(@) { Expr::BinaryOp(Box::new(x), Op::Gt, Box::new(y)) }
    x:@ _ ">=" _ y:(@) { Expr::BinaryOp(Box::new(x), Op::Gte, Box::new(y)) }
    x:@ _ "in" _ y:(@) { Expr::BinaryOp(Box::new(x), Op::In, Box::new(y)) }
    --
    x:@ _ "+" _ y:(@) { Expr::BinaryOp(Box::new(x),Op::Add, Box::new(y)) }
    x:@ _ "-" _ y:(@) { Expr::BinaryOp(Box::new(x),Op::Sub, Box::new(y)) }
    --
    x:mul_div() { x }
    --
    p:postfix() { p }
  }

  /// Postfix operations: property access and method calls with chaining
  rule postfix() -> Expr
    = base:atom() chain:(
        "." m:identifier() "(" args:((_ e:expression() _ {e}) ** ",") ")" { (m, Some(args)) }
        / "." p:identifier() { (p, None) }
      )* {
        let mut result = base;
        for (name, args) in chain {
          if let Some(args) = args {
            result = Expr::MethodCall(Box::new(result), name, args);
          } else {
            result = Expr::PropertyAccess(Box::new(result), name);
          }
        }
        result
      }

  rule atom() -> Expr
    = i:identifier() { Expr::Variable(i) }
    / i:string() { Expr::Value(Value::String(i)) }
    / i:number() { Expr::Value(Value::Number(i)) }
    / i:boolean_literal() { Expr::Value(i) }
    / "(" e:expression() ")" { e }
    / "-" e:atom() { Expr::UnaryOp(Op::Neg, Box::new(e)) }
    / "!" e:atom() { Expr::UnaryOp(Op::Not, Box::new(e)) }

  pub rule string() -> SmolStr
    = "\"" s:$(([^'"'] / "\\\"")*) "\"" {
      s.replace("\\\"", "\"").into()
    }
    / "'" s:$(([^'\''] / "\\''")*) "'" {
        s.replace("\\'", "'").into()
    }
  
  rule boolean_literal() -> Value
    = "true" { Value::Boolean(true) }
    / "false" { Value::Boolean(false) }


  pub rule expr_stmt() -> Stmt
    = e:expression() { Stmt::ExprStmt(Box::new(e)) }

  pub rule if_stmt() -> Stmt
    = "if" _ cond:expression() whitespace()? "then" whitespace()?
      then_body:statement()* whitespace()?
      else_part:else_clause()?
      "end" whitespace()? {
        Stmt::If(Box::new(cond), then_body, else_part)
      }

  pub rule else_clause() -> Vec<Stmt>
    = "else if" whitespace()? cond:expression() whitespace()? "then" whitespace()?
      then_body:statement()* whitespace()?
      else_part:else_clause()? whitespace()? {
        vec![Stmt::If(Box::new(cond), then_body, else_part)]
      }
    / "else" whitespace()? else_body:statement()* whitespace()? {
        else_body
      }

  pub rule assignment() -> Stmt
    = i:identifier() path:("." p:identifier() { p })+ _ "=" _ value:expression() {
        Stmt::PropertyAssignment(i, path, Box::new(value))
    }
    / i:identifier() _ op:compound_op() _ value:expression() {
        // Desugar compound assignment: x += 1 becomes x = x + 1
        let var_expr = Expr::Variable(i.clone());
        let combined = Expr::BinaryOp(Box::new(var_expr), op, Box::new(value));
        Stmt::Assignment(i, Box::new(combined))
    }
    / i:identifier() _ "=" _ value:expression() { Stmt::Assignment(i, Box::new(value)) }

  rule compound_op() -> Op
    = "+=" { Op::Add }
    / "-=" { Op::Sub }
    / "*=" { Op::Mul }
    / "/=" { Op::Div }
    / "%=" { Op::Mod }

  rule keyword()
  = ("if" / "then" / "else" / "end" / "true" / "false" / "in") !['a'..='z' | 'A'..='Z' | '0'..='9' | '_']

  rule identifier() -> SmolStr
  = !keyword() s:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*)
    { s.into() }

  rule number() -> Decimal
  = n:$(['0'..='9']+ ("." ['0'..='9']+)?) {?
      Decimal::from_str(n).map_err(|_| "invalid decimal")
  }

  rule whitespace()
    = ([' ' | '\t' | '\n' | '\r'] / comment())+

  rule comment()
    = "//" [^'\n']* "\n"?
    / "/*" (!"*/" [_])* "*/"

  rule _() = quiet!{([' ' | '\t'] / comment())*}

  // rule string_lit() -> Expr
  //     = "\"" s:$([^'"']*) "\""
  //       { Expr::String(s.to_string()) }
  }
);

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_binary_op() {
    let res = parser::binary_op("1 + 2 * 3 - 4 / 5");
    if let Err(e) = &res {
      println!("{}", e);
    }
    if let Ok(expr) = res {
      match expr {
        Expr::BinaryOp(left, Op::Add, right) => {
          assert!(matches!(*left, Expr::Value(_)));
          match *right {
            Expr::BinaryOp(left2, Op::Sub, right2) => {
              // Check 2 * 3
              match *left2 {
                Expr::BinaryOp(left3, Op::Mul, right3) => {
                  assert!(matches!(*left3, Expr::Value(_)));
                  assert!(matches!(*right3, Expr::Value(_)));
                }
                _ => panic!("Expected multiplication"),
              }
              // Check 4 / 5
              match *right2 {
                Expr::BinaryOp(left3, Op::Div, right3) => {
                  assert!(matches!(*left3, Expr::Value(_)));
                  assert!(matches!(*right3, Expr::Value(_)));
                }
                _ => panic!("Expected division"),
              }
            }
            _ => panic!("Expected subtraction"),
          }
        }
        _ => panic!("Expected addition at top level"),
      }
    } else {
      panic!("Failed to parse expression");
    }
  }

  #[test]
  fn test_function_call() {
    assert!(matches!(
      parser::expression("add(1, 2)"),
      Ok(Expr::FunctionCall(_, _))
    ));
  }

  #[test]
  fn test_var_decl() {
    let input = "x = 1";
    let res = parser::assignment(input);
    assert!(matches!(res, Ok(Stmt::Assignment(_, _))));
  }

  #[test]
  fn test_simple_arithmetic() {
    let input = "x = 1 + 2 * 3";
    let result = parser::program(input).unwrap();

    assert_eq!(result.len(), 1);
    if let Stmt::Assignment(name, expr) = &result[0] {
      assert_eq!(name, "x");
      if let Expr::BinaryOp(left, op, right) = expr.as_ref() {
        assert!(matches!(op, Op::Add));
        assert!(matches!(**left, Expr::Value(Value::Number(_))));
        if let Expr::BinaryOp(mul_left, mul_op, mul_right) = right.as_ref() {
          assert!(matches!(mul_op, Op::Mul));
          assert!(matches!(**mul_left, Expr::Value(Value::Number(_))));
          assert!(matches!(**mul_right, Expr::Value(Value::Number(_))));
        } else {
          panic!("Expected multiplication operation");
        }
      } else {
        panic!("Expected binary operation");
      }
    } else {
      panic!("Expected assignment statement");
    }
  }

  #[test]
  fn test_if_statement() {
    let input = "if x < 10 then y = x else y = 0 end";
    let result = parser::program(input).unwrap();

    assert_eq!(result.len(), 1);
    if let Stmt::If(condition, then_branch, else_branch) = &result[0] {
      // Check condition
      if let Expr::BinaryOp(left, op, right) = condition.as_ref() {
        assert!(matches!(op, Op::Lt));
        assert!(matches!(**left, Expr::Variable(_)));
        assert!(matches!(**right, Expr::Value(Value::Number(_))));
      } else {
        panic!("Expected binary operation in condition");
      }

      // Check then branch
      assert_eq!(then_branch.len(), 1);
      assert!(matches!(&then_branch[0], Stmt::Assignment(_, _)));

      // Check else branch
      assert!(else_branch.is_some());
      let else_branch = else_branch.as_ref().unwrap();
      assert_eq!(else_branch.len(), 1);
      assert!(matches!(&else_branch[0], Stmt::Assignment(_, _)));
    } else {
      panic!("Expected if statement");
    }
  }

  #[test]
  fn test_nested_function_calls() {
    let input = "result = max(min(a, b), abs(c))";
    let result = parser::program(input).unwrap();

    assert_eq!(result.len(), 1);
    if let Stmt::Assignment(name, expr) = &result[0] {
      assert_eq!(name, "result");
      if let Expr::FunctionCall(func_name, args) = expr.as_ref() {
        assert_eq!(func_name, "max");
        assert_eq!(args.len(), 2);

        // Check first argument (min call)
        if let Expr::FunctionCall(inner_func, inner_args) = &args[0] {
          assert_eq!(inner_func, "min");
          assert_eq!(inner_args.len(), 2);
        } else {
          panic!("Expected min function call");
        }

        // Check second argument (abs call)
        if let Expr::FunctionCall(inner_func, inner_args) = &args[1] {
          assert_eq!(inner_func, "abs");
          assert_eq!(inner_args.len(), 1);
        } else {
          panic!("Expected abs function call");
        }
      } else {
        panic!("Expected function call");
      }
    }
  }

  #[test]
  fn test_decimal_numbers() {
    let input = "x = 123.456";
    let result = parser::program(input).unwrap();

    if let Stmt::Assignment(_, expr) = &result[0] {
      if let Expr::Value(Value::Number(n)) = expr.as_ref() {
        assert_eq!(*n, Decimal::from_str("123.456").unwrap());
      } else {
        panic!("Expected decimal number");
      }
    }
  }

  #[test]
  fn test_complex_nested_if() {
    let input = r#"
            if x > 0 then
                if y > 0 then
                    result = x + y
                else
                    result = x - y
                end
            else
                result = 0
            end
        "#;
    let result = parser::program(input).unwrap();

    assert_eq!(result.len(), 1);
    if let Stmt::If(_, then_branch, else_branch) = &result[0] {
      // Check that then_branch contains another if statement
      assert_eq!(then_branch.len(), 1);
      assert!(matches!(&then_branch[0], Stmt::If(_, _, _)));

      // Check else branch
      assert!(else_branch.is_some());
      let else_branch = else_branch.as_ref().unwrap();
      assert_eq!(else_branch.len(), 1);
      assert!(matches!(&else_branch[0], Stmt::Assignment(_, _)));
    }
  }

  #[test]
  fn test_syntax_errors() {
    // Missing 'end' keyword
    assert!(parser::program("if x < 10 then y = x").is_err());

    // Invalid expression
    assert!(parser::program("x = 1 + * 2").is_err());
  }

  #[test]
  fn test_whitespace_handling() {
    let input1 = "x=1+2";
    let input2 = "x = 1 + 2";
    let input3 = "x   =   1   +   2";

    let result1 = parser::program(input1).unwrap();
    let result2 = parser::program(input2).unwrap();
    let result3 = parser::program(input3).unwrap();

    // All should produce equivalent ASTs
    assert_eq!(result1, result2);
    assert_eq!(result2, result3);
  }

  #[test]
  fn test_compound_assignment_parsing() {
    let input = "x += 5";
    let result = parser::program(input);
    println!("Result: {:?}", result);
    let result = result.unwrap();
    assert_eq!(result.len(), 1);
    if let Stmt::Assignment(name, expr) = &result[0] {
      assert_eq!(name, "x");
      // Should be desugared to x + 5
      if let Expr::BinaryOp(left, op, right) = expr.as_ref() {
        assert!(matches!(op, Op::Add));
        // left should be Variable("x")
        assert!(matches!(**left, Expr::Variable(_)));
        // right should be Number(5)
        assert!(matches!(**right, Expr::Value(Value::Number(_))));
      } else {
        panic!("Expected BinaryOp after desugaring, got {:?}", expr);
      }
    } else {
      panic!("Expected Assignment, got {:?}", result[0]);
    }
  }
}

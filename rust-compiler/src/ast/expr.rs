//! Expression AST nodes for Nature language

use crate::error::Location;
use serde::{Deserialize, Serialize};

/// All possible expressions in Nature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    /// Literal values
    Literal(Literal),
    /// Variable reference
    Variable(String),
    /// Function call
    Call(CallExpr),
    /// Method call
    MethodCall(MethodCallExpr),
    /// Binary operation
    Binary(BinaryExpr),
    /// Unary operation
    Unary(UnaryExpr),
    /// Conditional expression (ternary)
    Conditional(ConditionalExpr),
    /// Array/vector literal
    Array(ArrayExpr),
    /// Map literal
    Map(MapExpr),
    /// Tuple literal
    Tuple(TupleExpr),
    /// Struct literal
    Struct(StructExpr),
    /// Field access
    FieldAccess(FieldAccessExpr),
    /// Index access
    IndexAccess(IndexAccessExpr),
    /// Type cast
    Cast(CastExpr),
    /// Type assertion
    Assert(AssertExpr),
    /// Lambda/closure expression
    Lambda(LambdaExpr),
    /// Match expression
    Match(MatchExpr),
    /// Block expression
    Block(BlockExpr),
    /// If expression
    If(IfExpr),
    /// For expression
    For(ForExpr),
    /// While expression
    While(WhileExpr),
    /// Try expression
    Try(TryExpr),
    /// Select expression
    Select(SelectExpr),
    /// Go expression (goroutine)
    Go(GoExpr),
}

/// Literal values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    /// Integer literal
    Integer(i64),
    /// Float literal
    Float(f64),
    /// String literal
    String(String),
    /// Character literal
    Char(char),
    /// Boolean literal
    Boolean(bool),
    /// Null literal
    Null,
}

/// Function call expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallExpr {
    /// Function name or expression
    pub callee: Box<Expression>,
    /// Type arguments
    pub type_args: Vec<Type>,
    /// Function arguments
    pub arguments: Vec<Expression>,
    /// Location in source
    pub location: Location,
}

/// Method call expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodCallExpr {
    /// Object expression
    pub object: Box<Expression>,
    /// Method name
    pub method: String,
    /// Type arguments
    pub type_args: Vec<Type>,
    /// Method arguments
    pub arguments: Vec<Expression>,
    /// Location in source
    pub location: Location,
}

/// Binary operation expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryExpr {
    /// Left operand
    pub left: Box<Expression>,
    /// Binary operator
    pub operator: BinaryOp,
    /// Right operand
    pub right: Box<Expression>,
    /// Location in source
    pub location: Location,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    /// Addition operator (+)
    Add,        // +
    /// Subtraction operator (-)
    Sub,        // -
    /// Multiplication operator (*)
    Mul,        // *
    /// Division operator (/)
    Div,        // /
    /// Modulo operator (%)
    Mod,        // %
    /// Power operator (**)
    Pow,        // **
    
    // Bitwise
    /// Bitwise AND operator (&)
    BitAnd,     // &
    /// Bitwise OR operator (|)
    BitOr,      // |
    /// Bitwise XOR operator (^)
    BitXor,     // ^
    /// Left bit shift operator (<<)
    LeftShift,  // <<
    /// Right bit shift operator (>>)
    RightShift, // >>
    
    // Logical
    /// Logical AND operator (&&)
    LogicalAnd, // &&
    /// Logical OR operator (||)
    LogicalOr,  // ||
    
    // Comparison
    /// Equality operator (==)
    Equal,      // ==
    /// Inequality operator (!=)
    NotEqual,   // !=
    /// Less than operator (<)
    Less,       // <
    /// Less than or equal operator (<=)
    LessEqual,  // <=
    /// Greater than operator (>)
    Greater,    // >
    /// Greater than or equal operator (>=)
    GreaterEqual, // >=
    
    // Assignment
    /// Assignment operator (=)
    Assign,     // =
    /// Add and assign operator (+=)
    AddAssign,  // +=
    /// Subtract and assign operator (-=)
    SubAssign,  // -=
    /// Multiply and assign operator (*=)
    MulAssign,  // *=
    /// Divide and assign operator (/=)
    DivAssign,  // /=
    /// Modulo and assign operator (%=)
    ModAssign,  // %=
    /// Bitwise AND and assign operator (&=)
    BitAndAssign, // &=
    /// Bitwise OR and assign operator (|=)
    BitOrAssign,  // |=
    /// Bitwise XOR and assign operator (^=)
    BitXorAssign, // ^=
    /// Left shift and assign operator (<<=)
    LeftShiftAssign,  // <<=
    /// Right shift and assign operator (>>=)
    RightShiftAssign, // >>=
}

/// Unary operation expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnaryExpr {
    /// Unary operator
    pub operator: UnaryOp,
    /// Operand
    pub operand: Box<Expression>,
    /// Location in source
    pub location: Location,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    /// Logical not (!)
    Not,
    /// Bitwise not (~)
    BitNot,
    /// Unary minus (-)
    Neg,
    /// Unary plus (+)
    Pos,
    /// Dereference (*)
    Deref,
    /// Address of (&)
    AddrOf,
}

/// Conditional expression (ternary operator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalExpr {
    /// Condition
    pub condition: Box<Expression>,
    /// True branch
    pub true_expr: Box<Expression>,
    /// False branch
    pub false_expr: Box<Expression>,
    /// Location in source
    pub location: Location,
}

/// Array literal expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayExpr {
    /// Array elements
    pub elements: Vec<Expression>,
    /// Location in source
    pub location: Location,
}

/// Map literal expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapExpr {
    /// Map entries
    pub entries: Vec<MapEntry>,
    /// Location in source
    pub location: Location,
}

/// Map entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapEntry {
    /// Key
    pub key: Expression,
    /// Value
    pub value: Expression,
    /// Location in source
    pub location: Location,
}

/// Tuple literal expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TupleExpr {
    /// Tuple elements
    pub elements: Vec<Expression>,
    /// Location in source
    pub location: Location,
}

/// Struct literal expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructExpr {
    /// Struct type
    pub struct_type: Type,
    /// Field initializers
    pub fields: Vec<FieldInit>,
    /// Location in source
    pub location: Location,
}

/// Field initializer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInit {
    /// Field name
    pub name: String,
    /// Field value
    pub value: Expression,
    /// Location in source
    pub location: Location,
}

/// Field access expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAccessExpr {
    /// Object expression
    pub object: Box<Expression>,
    /// Field name
    pub field: String,
    /// Location in source
    pub location: Location,
}

/// Index access expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexAccessExpr {
    /// Object expression
    pub object: Box<Expression>,
    /// Index expression
    pub index: Box<Expression>,
    /// Location in source
    pub location: Location,
}

/// Type cast expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastExpr {
    /// Expression to cast
    pub expr: Box<Expression>,
    /// Target type
    pub target_type: Type,
    /// Location in source
    pub location: Location,
}

/// Type assertion expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertExpr {
    /// Expression to assert
    pub expr: Box<Expression>,
    /// Asserted type
    pub asserted_type: Type,
    /// Location in source
    pub location: Location,
}

/// Lambda/closure expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaExpr {
    /// Lambda parameters
    pub parameters: Vec<Parameter>,
    /// Return type
    pub return_type: Option<Type>,
    /// Lambda body
    pub body: Box<Expression>,
    /// Location in source
    pub location: Location,
}

/// Match expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchExpr {
    /// Expression to match
    pub expr: Box<Expression>,
    /// Match arms
    pub arms: Vec<MatchArm>,
    /// Location in source
    pub location: Location,
}

/// Match arm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    /// Pattern
    pub pattern: Pattern,
    /// Guard condition (if any)
    pub guard: Option<Expression>,
    /// Arm body
    pub body: Expression,
    /// Location in source
    pub location: Location,
}

/// Pattern for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    /// Literal pattern
    Literal(Literal),
    /// Variable pattern
    Variable(String),
    /// Wildcard pattern
    Wildcard,
    /// Tuple pattern
    Tuple(Vec<Pattern>),
    /// Struct pattern
    Struct(StructPattern),
    /// Array pattern
    Array(Vec<Pattern>),
    /// Type pattern
    Type(Type),
    /// Or pattern (alternative patterns)
    Or(Vec<Pattern>),
}

/// Struct pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructPattern {
    /// Struct type
    pub struct_type: Type,
    /// Field patterns
    pub fields: Vec<FieldPattern>,
    /// Location in source
    pub location: Location,
}

/// Field pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldPattern {
    /// Field name
    pub name: String,
    /// Field pattern
    pub pattern: Pattern,
    /// Location in source
    pub location: Location,
}

/// Block expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockExpr {
    /// Statements in the block
    pub statements: Vec<Statement>,
    /// Final expression (if any)
    pub final_expr: Option<Box<Expression>>,
    /// Location in source
    pub location: Location,
}

/// If expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfExpr {
    /// Condition
    pub condition: Box<Expression>,
    /// Then branch
    pub then_branch: Box<Expression>,
    /// Else branch (if any)
    pub else_branch: Option<Box<Expression>>,
    /// Location in source
    pub location: Location,
}

/// For expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForExpr {
    /// Loop variable
    pub variable: String,
    /// Iterable expression
    pub iterable: Box<Expression>,
    /// Loop body
    pub body: Box<Expression>,
    /// Location in source
    pub location: Location,
}

/// While expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhileExpr {
    /// Condition
    pub condition: Box<Expression>,
    /// Loop body
    pub body: Box<Expression>,
    /// Location in source
    pub location: Location,
}

/// Try expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TryExpr {
    /// Expression to try
    pub expr: Box<Expression>,
    /// Catch handlers
    pub catch_handlers: Vec<CatchHandler>,
    /// Location in source
    pub location: Location,
}

/// Catch handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchHandler {
    /// Exception type
    pub exception_type: Type,
    /// Exception variable
    pub variable: String,
    /// Handler body
    pub body: Expression,
    /// Location in source
    pub location: Location,
}

/// Select expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectExpr {
    /// Select cases
    pub cases: Vec<SelectCase>,
    /// Default case (if any)
    pub default_case: Option<Box<Expression>>,
    /// Location in source
    pub location: Location,
}

/// Select case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectCase {
    /// Channel expression
    pub channel: Expression,
    /// Case body
    pub body: Expression,
    /// Location in source
    pub location: Location,
}

/// Go expression (goroutine)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoExpr {
    /// Function call to execute in goroutine
    pub call: CallExpr,
    /// Location in source
    pub location: Location,
}

// Re-export types from types module
use crate::ast::types::{Type, Parameter};
use crate::ast::stmt::Statement;

impl Expression {
    /// Get the location of this expression
    pub fn location(&self) -> Location {
        match self {
            Expression::Literal(lit) => match lit {
                Literal::Integer(_) | Literal::Float(_) | Literal::String(_) | 
                Literal::Char(_) | Literal::Boolean(_) | Literal::Null => {
                    Location::new(0, 0, 0) // Literals don't have location info
                }
            },
            Expression::Variable(_) => Location::new(0, 0, 0),
            Expression::Call(expr) => expr.location,
            Expression::MethodCall(expr) => expr.location,
            Expression::Binary(expr) => expr.location,
            Expression::Unary(expr) => expr.location,
            Expression::Conditional(expr) => expr.location,
            Expression::Array(expr) => expr.location,
            Expression::Map(expr) => expr.location,
            Expression::Tuple(expr) => expr.location,
            Expression::Struct(expr) => expr.location,
            Expression::FieldAccess(expr) => expr.location,
            Expression::IndexAccess(expr) => expr.location,
            Expression::Cast(expr) => expr.location,
            Expression::Assert(expr) => expr.location,
            Expression::Lambda(expr) => expr.location,
            Expression::Match(expr) => expr.location,
            Expression::Block(expr) => expr.location,
            Expression::If(expr) => expr.location,
            Expression::For(expr) => expr.location,
            Expression::While(expr) => expr.location,
            Expression::Try(expr) => expr.location,
            Expression::Select(expr) => expr.location,
            Expression::Go(expr) => expr.location,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_expressions() {
        let int_lit = Expression::Literal(Literal::Integer(42));
        let str_lit = Expression::Literal(Literal::String("hello".to_string()));
        let bool_lit = Expression::Literal(Literal::Boolean(true));

        assert!(matches!(int_lit, Expression::Literal(Literal::Integer(42))));
        assert!(matches!(str_lit, Expression::Literal(Literal::String(_))));
        assert!(matches!(bool_lit, Expression::Literal(Literal::Boolean(true))));
    }

    #[test]
    fn test_binary_expression() {
        let left = Expression::Literal(Literal::Integer(10));
        let right = Expression::Literal(Literal::Integer(20));
        let location = Location::new(1, 10, 10);

        let binary = Expression::Binary(BinaryExpr {
            left: Box::new(left),
            operator: BinaryOp::Add,
            right: Box::new(right),
            location,
        });

        assert!(matches!(binary, Expression::Binary(_)));
    }

    #[test]
    fn test_function_call() {
        let callee = Expression::Variable("add".to_string());
        let args = vec![
            Expression::Literal(Literal::Integer(1)),
            Expression::Literal(Literal::Integer(2)),
        ];
        let location = Location::new(1, 1, 0);

        let call = Expression::Call(CallExpr {
            callee: Box::new(callee),
            type_args: vec![],
            arguments: args,
            location,
        });

        assert!(matches!(call, Expression::Call(_)));
    }
}

//! Statement AST nodes for Nature language

use crate::error::Location;
use crate::ast::expr::Expression;
use serde::{Deserialize, Serialize};

/// All possible statements in Nature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    /// Expression statement
    Expression(Expression),
    /// Variable declaration
    VariableDecl(VariableDeclStmt),
    /// Constant declaration
    ConstantDecl(ConstantDeclStmt),
    /// Assignment statement
    Assignment(AssignmentStmt),
    /// If statement
    If(IfStmt),
    /// For statement
    For(ForStmt),
    /// While statement
    While(WhileStmt),
    /// Match statement
    Match(MatchStmt),
    /// Return statement
    Return(ReturnStmt),
    /// Break statement
    Break(BreakStmt),
    /// Continue statement
    Continue(ContinueStmt),
    /// Try statement
    Try(TryStmt),
    /// Throw statement
    Throw(ThrowStmt),
    /// Select statement
    Select(SelectStmt),
    /// Go statement (goroutine)
    Go(GoStmt),
    /// Block statement
    Block(BlockStmt),
    /// Empty statement
    Empty,
}

/// Variable declaration statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDeclStmt {
    /// Variable name
    pub name: String,
    /// Variable type (if specified)
    pub var_type: Option<Type>,
    /// Initial value
    pub initializer: Option<Expression>,
    /// Is mutable
    pub mutable: bool,
    /// Location in source
    pub location: Location,
}

/// Constant declaration statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantDeclStmt {
    /// Constant name
    pub name: String,
    /// Constant type (if specified)
    pub const_type: Option<Type>,
    /// Constant value
    pub value: Expression,
    /// Location in source
    pub location: Location,
}

/// Assignment statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentStmt {
    /// Left-hand side (target)
    pub target: AssignmentTarget,
    /// Assignment operator
    pub operator: AssignmentOp,
    /// Right-hand side (value)
    pub value: Expression,
    /// Location in source
    pub location: Location,
}

/// Assignment target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignmentTarget {
    /// Simple variable
    Variable(String),
    /// Field access
    FieldAccess(FieldAccessTarget),
    /// Index access
    IndexAccess(IndexAccessTarget),
}

/// Field access target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAccessTarget {
    /// Object expression
    pub object: Expression,
    /// Field name
    pub field: String,
    /// Location in source
    pub location: Location,
}

/// Index access target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexAccessTarget {
    /// Object expression
    pub object: Expression,
    /// Index expression
    pub index: Expression,
    /// Location in source
    pub location: Location,
}

/// Assignment operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentOp {
    /// Simple assignment (=)
    Assign,
    /// Add and assign (+=)
    AddAssign,
    /// Subtract and assign (-=)
    SubAssign,
    /// Multiply and assign (*=)
    MulAssign,
    /// Divide and assign (/=)
    DivAssign,
    /// Modulo and assign (%=)
    ModAssign,
    /// Bitwise AND and assign (&=)
    BitAndAssign,
    /// Bitwise OR and assign (|=)
    BitOrAssign,
    /// Bitwise XOR and assign (^=)
    BitXorAssign,
    /// Left shift and assign (<<=)
    LeftShiftAssign,
    /// Right shift and assign (>>=)
    RightShiftAssign,
}

/// If statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfStmt {
    /// Condition
    pub condition: Expression,
    /// Then branch
    pub then_branch: Box<Statement>,
    /// Else branch (if any)
    pub else_branch: Option<Box<Statement>>,
    /// Location in source
    pub location: Location,
}

/// For statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForStmt {
    /// Loop variable
    pub variable: String,
    /// Iterable expression
    pub iterable: Expression,
    /// Loop body
    pub body: Box<Statement>,
    /// Location in source
    pub location: Location,
}

/// While statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhileStmt {
    /// Condition
    pub condition: Expression,
    /// Loop body
    pub body: Box<Statement>,
    /// Location in source
    pub location: Location,
}

/// Match statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchStmt {
    /// Expression to match
    pub expr: Expression,
    /// Match arms
    pub arms: Vec<MatchArmStmt>,
    /// Location in source
    pub location: Location,
}

/// Match arm for statements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArmStmt {
    /// Pattern
    pub pattern: Pattern,
    /// Guard condition (if any)
    pub guard: Option<Expression>,
    /// Arm body
    pub body: Statement,
    /// Location in source
    pub location: Location,
}

/// Return statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnStmt {
    /// Return value (if any)
    pub value: Option<Expression>,
    /// Location in source
    pub location: Location,
}

/// Break statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakStmt {
    /// Break label (if any)
    pub label: Option<String>,
    /// Location in source
    pub location: Location,
}

/// Continue statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinueStmt {
    /// Continue label (if any)
    pub label: Option<String>,
    /// Location in source
    pub location: Location,
}

/// Try statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TryStmt {
    /// Try block
    pub try_block: Box<Statement>,
    /// Catch handlers
    pub catch_handlers: Vec<CatchHandlerStmt>,
    /// Location in source
    pub location: Location,
}

/// Catch handler for statements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchHandlerStmt {
    /// Exception type
    pub exception_type: Type,
    /// Exception variable
    pub variable: String,
    /// Handler body
    pub body: Statement,
    /// Location in source
    pub location: Location,
}

/// Throw statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrowStmt {
    /// Exception to throw
    pub exception: Expression,
    /// Location in source
    pub location: Location,
}

/// Select statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectStmt {
    /// Select cases
    pub cases: Vec<SelectCaseStmt>,
    /// Default case (if any)
    pub default_case: Option<Statement>,
    /// Location in source
    pub location: Location,
}

/// Select case for statements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectCaseStmt {
    /// Channel expression
    pub channel: Expression,
    /// Case body
    pub body: Statement,
    /// Location in source
    pub location: Location,
}

/// Go statement (goroutine)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoStmt {
    /// Function call to execute in goroutine
    pub call: CallExpr,
    /// Location in source
    pub location: Location,
}

/// Block statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStmt {
    /// Statements in the block
    pub statements: Vec<Statement>,
    /// Location in source
    pub location: Location,
}

// Re-export types from other modules
use crate::ast::types::Type;
use crate::ast::expr::{Pattern, CallExpr};

impl Statement {
    /// Get the location of this statement
    pub fn location(&self) -> Location {
        match self {
            Statement::Expression(expr) => expr.location(),
            Statement::VariableDecl(stmt) => stmt.location,
            Statement::ConstantDecl(stmt) => stmt.location,
            Statement::Assignment(stmt) => stmt.location,
            Statement::If(stmt) => stmt.location,
            Statement::For(stmt) => stmt.location,
            Statement::While(stmt) => stmt.location,
            Statement::Match(stmt) => stmt.location,
            Statement::Return(stmt) => stmt.location,
            Statement::Break(stmt) => stmt.location,
            Statement::Continue(stmt) => stmt.location,
            Statement::Try(stmt) => stmt.location,
            Statement::Throw(stmt) => stmt.location,
            Statement::Select(stmt) => stmt.location,
            Statement::Go(stmt) => stmt.location,
            Statement::Block(stmt) => stmt.location,
            Statement::Empty => Location::new(0, 0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_statement() {
        let expr = Expression::Literal(crate::ast::expr::Literal::Integer(42));
        let stmt = Statement::Expression(expr);
        
        assert!(matches!(stmt, Statement::Expression(_)));
    }

    #[test]
    fn test_variable_declaration() {
        let location = Location::new(1, 1, 0);
        let stmt = Statement::VariableDecl(VariableDeclStmt {
            name: "x".to_string(),
            var_type: None,
            initializer: Some(Expression::Literal(crate::ast::expr::Literal::Integer(42))),
            mutable: true,
            location,
        });

        assert!(matches!(stmt, Statement::VariableDecl(_)));
    }

    #[test]
    fn test_assignment_statement() {
        let location = Location::new(1, 1, 0);
        let stmt = Statement::Assignment(AssignmentStmt {
            target: AssignmentTarget::Variable("x".to_string()),
            operator: AssignmentOp::Assign,
            value: Expression::Literal(crate::ast::expr::Literal::Integer(42)),
            location,
        });

        assert!(matches!(stmt, Statement::Assignment(_)));
    }

    #[test]
    fn test_return_statement() {
        let location = Location::new(1, 1, 0);
        let stmt = Statement::Return(ReturnStmt {
            value: Some(Expression::Literal(crate::ast::expr::Literal::Integer(42))),
            location,
        });

        assert!(matches!(stmt, Statement::Return(_)));
    }
}

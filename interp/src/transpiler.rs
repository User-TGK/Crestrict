/// Transpiler from C11 sources to Clight.
///
/// The provided transpilation functions convert a subset of the `clang_c` AST
/// into a Clight AST. The unimplemented macro is used at some places from which we
/// cannot recover. An error message is given most of the time, indicating whether
/// unsupported features or invalid syntax were used.
///
use lang_c::ast::{
    ArraySize, BinaryOperator, BinaryOperatorExpression, BlockItem, Constant, Declaration,
    DeclarationSpecifier, Declarator, DeclaratorKind, DerivedDeclarator, Expression,
    ExternalDeclaration, ForInitializer, ForStatement, Initializer, ParameterDeclaration,
    PointerQualifier, SpecifierQualifier, Statement, TypeQualifier, TypeSpecifier, UnaryOperator,
    WhileStatement,
};
use lang_c::{
    driver::{parse, Config},
    // print::Printer,
    span::Node,
    // visit::Visit,
};

use crate::ast::{
    BinOp, Dcl, Expr as CExpr, FunDef, Program, SimpleType, Statement as CStatement, Type,
    TypeQualifier as CTypeQualifier, UnOp,
};

/// Helper trait to implement the conversion of a fully-annotated C11
/// Abstract Syntax Tree to a Clight Abstract Syntax Tree.
trait Elaborate {
    /// The Clight output node
    type CNode;

    /// Conversion of a C11 node to a CNode or an Error String
    fn elaborate(&self) -> Result<Self::CNode, String>;
}

/// Function parameter declarations
impl Elaborate for ParameterDeclaration {
    type CNode = Dcl;

    fn elaborate(&self) -> Result<Self::CNode, String> {
        let base_ty = specifiers_to_type(&self.specifiers)?;
        let mut ty = base_ty;
        let identifier;

        let declarator = &self
            .declarator
            .as_ref()
            .ok_or(String::from("Parameter declarations must be named"))?
            .node;

        if let DeclaratorKind::Identifier(iden) = &declarator.kind.node {
            identifier = iden.node.name.clone();
        } else {
            return Err(String::from(format!(
                "Declarator with unsupported kind (1)'{:?}'",
                &self.declarator
            )));
        }

        for derived in &declarator.derived {
            match &derived.node {
                DerivedDeclarator::Pointer(p) => {
                    let mut qualifiers: Vec<CTypeQualifier> = vec![];

                    if p.len() == 1 {
                        if let PointerQualifier::TypeQualifier(q) = &p[0].node {
                            if let TypeQualifier::Restrict = &q.node {
                                qualifiers.push(CTypeQualifier::Restrict);
                            } else {
                                unimplemented!("{:?}", q.node)
                            }
                        } else {
                            unimplemented!("{:?}", p[0].node)
                        }
                    }

                    ty = Type::new(SimpleType::Ptr(Box::new(ty))).with_qualifiers(qualifiers);
                }
                DerivedDeclarator::Array(a) => {
                    let size = match &a.node.size {
                        ArraySize::VariableExpression(e) => {
                            if let Expression::Constant(c) = &e.node {
                                match &c.node {
                                    Constant::Integer(i) => {
                                        i.number.parse::<i32>().expect(&format!(
                                            "Failure in parsing {} to Clight integer constant",
                                            i.number
                                        ))
                                    }
                                    _ => unimplemented!("{:?}", c.node),
                                }
                            } else {
                                unimplemented!("{:?}", e.node)
                            }
                        }
                        _ => unimplemented!("{:?}", a.node.size),
                    };

                    ty = Type::new(SimpleType::Array(Box::new(ty), size));
                }
                _ => {}
            }
        }

        Ok((identifier, ty.parameter_conversion(), None))
    }
}

/// TypeSpecifier into a Clight type.
impl Elaborate for TypeSpecifier {
    type CNode = Type;

    fn elaborate(&self) -> Result<Self::CNode, String> {
        match self {
            TypeSpecifier::Void => Ok(Type::new(SimpleType::Void)),
            TypeSpecifier::Int => Ok(Type::new(SimpleType::I32)),
            _ => Err(format!(
                "Unimplemented conversion from type specifier {:?}",
                self
            )),
        }
    }
}

/// Try to convert a declarator to an identifier.
///
/// Returns `None` if the declarator was not of the Identifier kind.
///
fn to_ident(declarator: &Node<Declarator>) -> Option<String> {
    if let DeclaratorKind::Identifier(iden) = &declarator.node.kind.node {
        Some(iden.node.name.clone())
    } else {
        None
    }
}

// Helper function to 'elaborate' a declarator node, optionally setting the identifier
// and pushing a `DeclaratorItemKind` to the stack of decarators from which the type
// will be composed.
fn declarator_elaborate(
    declarator: &Declarator,
    identifier: &mut Option<String>,
    declarators: &mut Vec<DeclaratorItemKind>,
) -> Result<(), String> {
    match &declarator.kind.node {
        DeclaratorKind::Abstract => {
            // declarator_elaborate(&decl.node, identifier, declarators)?;
            // return Err(String::from(format!(
            //     "Declarator with unsupported kind (2)'{:?}'",
            //     declarator
            // )))
        }
        DeclaratorKind::Identifier(iden) => {
            *identifier = Some(iden.node.name.clone());
        }
        DeclaratorKind::Declarator(decl) => {
            declarator_elaborate(&decl.node, identifier, declarators)?;
        }
    }

    // Reverse
    for derived in declarator.derived.iter().rev() {
        match &derived.node {
            DerivedDeclarator::Pointer(p) => {
                let mut qualifiers: Vec<CTypeQualifier> = vec![];

                if p.len() == 1 {
                    if let PointerQualifier::TypeQualifier(q) = &p[0].node {
                        if let TypeQualifier::Restrict = &q.node {
                            qualifiers.push(CTypeQualifier::Restrict);
                        } else {
                            unimplemented!("{:?}", q.node)
                        }
                    } else {
                        unimplemented!("{:?}", p[0].node)
                    }
                }

                declarators.push(DeclaratorItemKind::Pointer(qualifiers));
            }
            DerivedDeclarator::Array(a) => {
                let size = match &a.node.size {
                    ArraySize::VariableExpression(e) => {
                        if let Expression::Constant(c) = &e.node {
                            match &c.node {
                                Constant::Integer(i) => i.number.parse::<i32>().expect(&format!(
                                    "Failure in parsing {} to Clight integer constant",
                                    i.number
                                )),
                                _ => unimplemented!("{:?}", c.node),
                            }
                        } else {
                            unimplemented!("{:?}", e.node)
                        }
                    }
                    _ => unimplemented!("{:?}", a.node.size),
                };
                declarators.push(DeclaratorItemKind::Array(size));
            }
            _ => {}
        }
    }

    Ok(())
}

enum DeclaratorItemKind {
    Pointer(Vec<CTypeQualifier>),
    Array(i32),
}

/// C11 Declaration to Clight Dcl.
impl Elaborate for Declaration {
    type CNode = Dcl;

    fn elaborate(&self) -> Result<Self::CNode, String> {
        // Store declarators so we can create the correct Clight type
        let mut declarators: Vec<DeclaratorItemKind> = Vec::new();

        let base_ty = specifiers_to_type(&self.specifiers)?;
        let mut ty = base_ty;
        let mut identifier = None;
        let mut init_expr = None;

        if self.declarators.len() > 1 {
            return Err(format!(
                "Only a single init-decl is supported, got: {:?}",
                self.declarators
            ));
        }

        let init_decl = &self.declarators[0].node;

        declarator_elaborate(
            &init_decl.declarator.node,
            &mut identifier,
            &mut declarators,
        )?;

        if let Some(expr) = &init_decl.initializer {
            match &expr.node {
                Initializer::Expression(e) => {
                    init_expr = Some(e.node.elaborate()?);
                }
                // Optional TODO: simple list initializers
                // Initializer::List(items) => {}
                _ => {
                    unimplemented!("{:?}", expr.node)
                }
            }
        }

        for declarator in declarators.into_iter().rev() {
            match declarator {
                DeclaratorItemKind::Pointer(qualifiers) => {
                    ty = Type::new(SimpleType::Ptr(Box::new(ty))).with_qualifiers(qualifiers);
                }
                DeclaratorItemKind::Array(size) => {
                    ty = Type::new(SimpleType::Array(Box::new(ty), size));
                }
            }
        }

        Ok((identifier.expect("Identifier not set"), ty, init_expr))
    }
}

fn specifier_qualifier_to_type(specifiers: &Vec<Node<SpecifierQualifier>>) -> Result<Type, String> {
    let mut ty = None;

    for specifier in specifiers {
        if let SpecifierQualifier::TypeSpecifier(ty_specifier) = &specifier.node {
            let ty_specifier = &ty_specifier.node;
            if ty.is_none() {
                ty = Some(ty_specifier.elaborate()?);
            } else {
                log::error!("Multiple types found in the provided specifiers qualifiers");
            }
        }
    }

    ty.ok_or(String::from(
        "No type could be extracted for the provided specifiers",
    ))
}

/// Extract a type from a set of declaration specifiers.
///
/// Returns an error message if no type was found in the provided specifiers.
/// All qualifiers except `restrict` are currently ommitted.
///
fn specifiers_to_type(specifiers: &Vec<Node<DeclarationSpecifier>>) -> Result<Type, String> {
    let mut ty = None;

    for specifier in specifiers {
        if let DeclarationSpecifier::TypeSpecifier(ty_specifier) = &specifier.node {
            let ty_specifier = &ty_specifier.node;
            if ty.is_none() {
                ty = Some(ty_specifier.elaborate()?);
            } else {
                // TODO: in what cases can this situation occur?
                log::error!("Multiple types found in the provided specifiers");
            }
        }
    }

    ty.ok_or(String::from(
        "No type could be extracted for the provided specifiers",
    ))
}

/// Try to convert an expression into a Clight expression.
impl Elaborate for Expression {
    type CNode = CExpr;

    fn elaborate(&self) -> Result<Self::CNode, String> {
        match self {
            Expression::Identifier(iden) => Ok(CExpr::EId(
                iden.node.name.clone(),
                Type::new(SimpleType::Undef),
            )),

            Expression::Call(call_expr) => Ok(CExpr::ECall(
                Box::new(call_expr.node.callee.node.elaborate()?),
                call_expr
                    .node
                    .arguments
                    .iter()
                    .map(|e: &Node<Expression>| e.node.elaborate().unwrap())
                    .collect(),
                Type::new(SimpleType::Undef),
            )),

            Expression::Constant(constant) => match &constant.node {
                Constant::Integer(i) => Ok(CExpr::EConstInt(
                    i.number.parse::<i32>().expect(&format!(
                        "Failure in parsing {} to Clight integer constant",
                        i.number
                    )),
                    Type::new(SimpleType::Undef),
                )),
                _ => unimplemented!("{:?}", constant.node),
            },

            Expression::SizeOfTy(ty) => {
                let node = &ty.node;
                let mut ty = specifier_qualifier_to_type(&node.0.node.specifiers)?;

                if let Some(declarator) = &node.0.node.declarator {
                    let mut declarators = vec![];

                    declarator_elaborate(&declarator.node, &mut None, &mut declarators)?;

                    for declarator in declarators.into_iter().rev() {
                        match declarator {
                            DeclaratorItemKind::Pointer(qualifiers) => {
                                ty = Type::new(SimpleType::Ptr(Box::new(ty)))
                                    .with_qualifiers(qualifiers);
                            }
                            DeclaratorItemKind::Array(size) => {
                                ty = Type::new(SimpleType::Array(Box::new(ty), size));
                            }
                        }
                    }
                }

                Ok(CExpr::ESizeOf(ty, Type::new(SimpleType::Undef)))
            }

            Expression::UnaryOperator(unop_expr) => {
                let operand = Box::new(unop_expr.node.operand.node.elaborate()?);

                match &unop_expr.node.operator.node {
                    UnaryOperator::Address => {
                        return Ok(CExpr::EAddrOf(operand, Type::new(SimpleType::Undef)))
                    }
                    UnaryOperator::Indirection => {
                        return Ok(CExpr::EDeref(operand, Type::new(SimpleType::Undef)))
                    }
                    _ => {}
                }

                let op = match &unop_expr.node.operator.node {
                    UnaryOperator::Negate => UnOp::ONotBool, // boolean negation (! in C)
                    UnaryOperator::Complement => UnOp::ONotInt, // integer complement (~ in C)
                    UnaryOperator::Minus => UnOp::ONeg,      // opposite (unary -)
                    _ => unimplemented!("Operator {:?}", unop_expr.node.operator.node),
                };

                Ok(CExpr::EUnOp(op, operand, Type::new(SimpleType::Undef)))
            }

            Expression::BinaryOperator(binop_expr) => {
                if let BinaryOperator::Index = &binop_expr.node.operator.node {
                    let res = CExpr::EDeref(
                        Box::new(CExpr::EBinOp(
                            BinOp::OAdd,
                            Box::new(binop_expr.node.lhs.node.elaborate()?),
                            Box::new(binop_expr.node.rhs.node.elaborate()?),
                            Type::new(SimpleType::Undef),
                        )),
                        Type::new(SimpleType::Undef),
                    );

                    return Ok(res);
                }

                let op = match &binop_expr.node.operator.node {
                    BinaryOperator::Plus => BinOp::OAdd,
                    BinaryOperator::Minus => BinOp::OSub,
                    BinaryOperator::Multiply => BinOp::OMul,
                    BinaryOperator::Divide => BinOp::ODiv,
                    BinaryOperator::Modulo => BinOp::OMod,
                    BinaryOperator::BitwiseAnd => BinOp::OAnd,
                    BinaryOperator::BitwiseOr => BinOp::OOr,
                    BinaryOperator::BitwiseXor => BinOp::OXor,
                    BinaryOperator::ShiftLeft => BinOp::OShl,
                    BinaryOperator::ShiftRight => BinOp::OShr,
                    BinaryOperator::Equals => BinOp::OEq,
                    BinaryOperator::NotEquals => BinOp::ONe,
                    BinaryOperator::Less => BinOp::OLt,
                    BinaryOperator::Greater => BinOp::OGt,
                    BinaryOperator::LessOrEqual => BinOp::OLe,
                    BinaryOperator::GreaterOrEqual => BinOp::OGe,
                    _ => {
                        unimplemented!("Unimplemented binary operator {:?}", binop_expr);
                    }
                };

                Ok(CExpr::EBinOp(
                    op,
                    Box::new(binop_expr.node.lhs.node.elaborate()?),
                    Box::new(binop_expr.node.rhs.node.elaborate()?),
                    Type::new(SimpleType::Undef),
                ))
            }
            _ => {
                unimplemented!("Unimplemented expression {:?}", self);
            }
        }
    }
}

fn map_args(args: &[Node<Expression>]) -> Result<Vec<CExpr>, String> {
    args.iter().map(|e| e.node.elaborate()).collect()
}

impl Elaborate for WhileStatement {
    type CNode = (Vec<Dcl>, Option<CStatement>);

    fn elaborate(&self) -> Result<Self::CNode, String> {
        if let (dcls, Some(s2)) = self.statement.node.elaborate()? {
            Ok((
                dcls,
                Some(CStatement::SWhile(
                    self.expression.node.elaborate().unwrap(),
                    Box::new(s2),
                )),
            ))
        } else {
            return Err(String::from(
                "While condition did not evaluate to statement",
            ));
        }
    }
}

impl Elaborate for ForStatement {
    type CNode = (Vec<Dcl>, Option<CStatement>);

    fn elaborate(&self) -> Result<Self::CNode, String> {
        let mut dcls: Vec<Dcl> = Vec::new();

        let s1 = match &self.initializer.node {
            ForInitializer::Empty => CStatement::SSkip,
            ForInitializer::Expression(e) => {
                if let (mut dcls_1, Some(s1)) =
                    Statement::Expression(Some(e.clone())).elaborate()?
                {
                    dcls.append(&mut dcls_1);
                    s1
                } else {
                    unimplemented!("Unexpected s1")
                }
            }
            ForInitializer::Declaration(d) => {
                let (id, ty, e) = d.node.elaborate()?;
                dcls.push((id.clone(), ty, None));
                CStatement::SAssign(
                    CExpr::EId(id, Type::new(SimpleType::Undef)),
                    e.ok_or(String::from(
                        "For loop initializer declaration is missing an initial value",
                    ))?,
                )
            }
            _ => {
                unimplemented!("Unimplemented {:?}", &self.initializer.node)
            }
        };

        let e = match &self.condition {
            Some(c) => c.node.elaborate()?,
            None => {
                unimplemented!("For condition may not be empty")
            }
        };

        let s2 = match &self.step {
            Some(step_expr) => Statement::Expression(Some(step_expr.clone()))
                .elaborate()?
                .1
                .ok_or(String::from("Unable to transpile for loop step"))?,
            None => CStatement::SSkip,
        };

        let s = self
            .statement
            .node
            .elaborate()?
            .1
            .ok_or("Unable to transpile for loop body")?;

        Ok((
            dcls,
            Some(CStatement::SFor(Box::new(s1), e, Box::new(s2), Box::new(s))),
        ))
    }
}

impl Elaborate for BinaryOperatorExpression {
    type CNode = (Vec<Dcl>, Option<CStatement>);

    fn elaborate(&self) -> Result<Self::CNode, String> {
        match &self.operator.node {
            // `lhs = rhs`
            BinaryOperator::Assign => {
                // Check if rhs is a call (and if so, we emit a call expr)
                if let Expression::Call(call) = &self.rhs.node {
                    let e = call.node.callee.node.elaborate()?;

                    // Check if it is a malloc invocation
                    if let CExpr::EId(ref id, _) = e {
                        if id == "malloc" {
                            if call.node.arguments.len() != 1 {
                                return Err(String::from(
                                    "Call to malloc takes exactly 1 argument (size)",
                                ));
                            }

                            let mut args: Vec<CExpr> = call
                                .node
                                .arguments
                                .iter()
                                .map(|e| e.node.elaborate().unwrap())
                                .collect();

                            return Ok((
                                Vec::new(),
                                Some(CStatement::SMalloc(
                                    self.lhs.node.elaborate()?,
                                    args.remove(0),
                                )),
                            ));
                        }
                    }
                }
                Ok((
                    Vec::new(),
                    Some(CStatement::SAssign(
                        self.lhs.node.elaborate()?,
                        self.rhs.node.elaborate()?,
                    )),
                ))
            }
            _ => {
                unimplemented!("{:?}", self.operator.node)
            }
        }
    }
}

/// Try to convert a statement into a Clight statement.
///
/// This converts a subset of C11 statements into a Clight statement.
/// We also cover the impure expressions `lhs = rhs` (assignment) and function calls.
///
/// As variable declarations are present inside compound statements, the return
/// value also contains a list of local variable declarations (i.e. function scope).
/// We currently do not verify that these declarations only occur at the start of
/// such a block (TODO).
impl Elaborate for Statement {
    type CNode = (Vec<Dcl>, Option<CStatement>);

    fn elaborate(&self) -> Result<Self::CNode, String> {
        match self {
            Statement::Compound(items) => {
                let mut decls = vec![];
                let mut statements: Vec<CStatement> = vec![];

                for block in items {
                    let block = &block.node;
                    match block {
                        // NOTE: we don't enforce in the transpile step that variable declarations are at the start,
                        // but do assume that this precondition is fulfilled in the interpreter itself.
                        BlockItem::Declaration(decl) => {
                            decls.push(decl.node.elaborate()?);
                        }
                        BlockItem::StaticAssert(_) => log::warn!("Static assert discarded"),
                        BlockItem::Statement(s) => {
                            if let (mut dcl, Some(ss)) = s.node.elaborate()? {
                                decls.append(&mut dcl);
                                statements.push(ss)
                            } else {
                                return Err(String::from(
                                    "Recursive call to parsing a statement did not return a statement",
                                ));
                            }
                        }
                    }
                }

                let statement_res = match statements.len() {
                    0 => None,
                    1 => Some(statements[0].clone()),
                    _ => {
                        let mut statement = statements[0].clone();
                        for i in 1..(statements.len()) {
                            statement = CStatement::SSequence(
                                Box::new(statement),
                                Box::new(statements[i].clone()),
                            );
                        }

                        Some(statement)
                    }
                };

                Ok((decls, statement_res))
            }
            Statement::Break => Ok((Vec::new(), Some(CStatement::SBreak))),
            Statement::Continue => Ok((Vec::new(), Some(CStatement::SContinue))),
            Statement::While(s) => s.node.elaborate(),
            Statement::For(for_node) => for_node.node.elaborate(),
            Statement::Expression(Some(e)) => {
                match &e.node {
                    Expression::UnaryOperator(un_op) => {
                        let node = match &un_op.node.operator.node {
                            UnaryOperator::PreIncrement | UnaryOperator::PostIncrement => {
                                let lvalue_expr = un_op.node.operand.node.elaborate()?;
                                CStatement::SAssign(
                                    lvalue_expr.clone(),
                                    CExpr::EBinOp(
                                        BinOp::OAdd,
                                        Box::new(lvalue_expr),
                                        Box::new(CExpr::EConstInt(1, Type::new(SimpleType::Undef))),
                                        Type::new(SimpleType::Undef),
                                    ),
                                )
                            }

                            _ => unimplemented!("{:?}", un_op.node.operator.node),
                        };

                        Ok((Vec::new(), Some(node)))
                    }

                    // Filter out assignment + function call (non-pure expressions)
                    Expression::BinaryOperator(bin_op) => bin_op.node.elaborate(),
                    Expression::Call(call) => {
                        let e = call.node.callee.node.elaborate()?;

                        let statement = match e {
                            CExpr::EId(ref value, _) if value == "free" => {
                                let mut args = map_args(&call.node.arguments)?; // assert!(args.len() == 1)
                                CStatement::SFree(args.remove(0))
                            }
                            CExpr::EId(ref value, _) if value == "printf" => {
                                if let Expression::StringLiteral(format_str) =
                                    &call.node.arguments[0].node
                                {
                                    let args = map_args(&call.node.arguments[1..])?;
                                    CStatement::SPrint(format_str.node[0].to_owned(), args)
                                } else {
                                    log::error!("First argument of printf must be a format string",);
                                    unimplemented!("{:?}", call.node.arguments[0].node)
                                }
                            }
                            _ => CStatement::SCall(e, map_args(&call.node.arguments)?),
                        };

                        Ok((Vec::new(), Some(statement)))
                    }
                    _ => {
                        unimplemented!("{:?}", e.node)
                    }
                }
            }
            Statement::If(if_statement) => {
                let if_statement = &if_statement.node;
                let else_case = match &if_statement.else_statement {
                    Some(s) => s
                        .node
                        .elaborate()?
                        .1
                        .ok_or(String::from("Else branch statement transpilation failed"))?,
                    None => CStatement::SSkip,
                };

                Ok((
                    Vec::new(),
                    Some(CStatement::SIfThenElse(
                        if_statement.condition.node.elaborate()?,
                        Box::new(
                            if_statement
                                .then_statement
                                .node
                                .elaborate()?
                                .1
                                .ok_or(String::from("If branch statement transpilation failed"))?,
                        ),
                        Box::new(else_case),
                    )),
                ))
            }
            Statement::Return(e) => match e {
                None => Ok((Vec::new(), Some(CStatement::SReturn(None)))),
                Some(e) => Ok((
                    Vec::new(),
                    Some(CStatement::SReturn(Some(e.node.elaborate().unwrap()))),
                )),
            },
            _ => {
                unimplemented!("{:?}", self)
            }
        }
    }
}

#[derive(Default)]
pub struct Transpiler {}

impl Transpiler {
    pub fn transpile(&self, source_file: &str) -> Result<Program, String> {
        let mut config = Config::default();

        // This allows doing a check based on C compiler defines;
        // we conditionally include the <stdio.h> library based on this.
        config.cpp_options.push(String::from("-undef"));

        let parse = parse(&config, source_file).map_err(|e| e.to_string())?;
        let mut program = Program::default();

        for decl in &parse.unit.0 {
            match &decl.node {
                ExternalDeclaration::Declaration(decl) => {
                    program.var_decls.push(decl.node.elaborate()?);
                }
                ExternalDeclaration::StaticAssert(_a) => unimplemented!("{:?}", decl.node),
                // C11 6.9.1
                // Note, we don't support K&R style function definitions
                // https://jameshfisher.com/2016/11/27/c-k-and-r/
                ExternalDeclaration::FunctionDefinition(fun_decl) => {
                    let fun_decl = &fun_decl.node;
                    let fun_name = to_ident(&fun_decl.declarator)
                        .ok_or("Anonymous functions are not supported")?;

                    let mut return_type = specifiers_to_type(&fun_decl.specifiers)?;
                    let mut parameters: Vec<Dcl> = vec![];

                    // Params
                    let declarator = &fun_decl.declarator.node;
                    for derived in &declarator.derived {
                        match &derived.node {
                            DerivedDeclarator::Function(fun_decl) => {
                                for param in &fun_decl.node.parameters {
                                    parameters.push(param.node.elaborate()?);
                                }
                            }
                            // can return types be restrict annotated?
                            DerivedDeclarator::Pointer(_pointer_qualifiers) => {
                                return_type = Type::new(SimpleType::Ptr(Box::new(return_type)));
                            }
                            _ => {}
                        }
                    }

                    let (local_var_decls, body) = fun_decl.statement.node.elaborate()?;

                    let fun_def = FunDef {
                        return_type,
                        parameters,
                        local_var_decls,
                        body: body
                            .ok_or(format!("Failure in parsing function body of {}", fun_name))?,
                    };

                    program.fun_decls.push((fun_name, fun_def));
                }
            }
        }

        Ok(program)
    }
}

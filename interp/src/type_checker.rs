use crate::ast::*;

use std::collections::HashMap;

pub type TypingContextInstance = HashMap<Ident, Type>;

/// Context
pub struct TypingContext {
    /// Local scope context (function block)
    pub local: TypingContextInstance,

    /// Global scope context, variables (translation unit)
    pub global: TypingContextInstance,

    /// Global scope context, functions (translation unit)
    pub functions: TypingContextInstance,
}

impl TypingContext {
    pub fn new() -> Self {
        Self {
            local: TypingContextInstance::new(),
            global: TypingContextInstance::new(),
            functions: TypingContextInstance::new(),
        }
    }

    pub fn get_ty(&self, id: &Ident) -> Option<Type> {
        if let Some(ty) = self.local.get(id) {
            Some(ty.clone())
        } else {
            // Note: this implies that global variables and function names may not duplicate
            if let Some(ty) = self.functions.get(id) {
                Some(ty.clone())
            } else {
                self.global.get(id).cloned()
            }
        }
    }
}

pub trait TypeCheck {
    fn type_check(&mut self, context: &mut TypingContext) -> Result<(), String>;
}

impl TypeCheck for Expr {
    fn type_check(&mut self, context: &mut TypingContext) -> Result<(), String> {
        match self {
            Expr::EConstInt(_, ty) => {
                *ty = Type::new(SimpleType::I32);
            }
            Expr::ESizeOf(_, ty) => {
                *ty = Type::new(SimpleType::I32);
            }
            Expr::ECall(e1, es, ty) => {
                e1.type_check(context)?;

                if let SimpleType::Function(ref param_types, return_ty) = &e1.type_of().ty {
                    *ty = *return_ty.clone();

                    // Check return type
                    // if let Some(e1) = e1 {
                    //     e1.type_check(context)?;

                    //     let lhs_ty = e1.type_of();
                    //     if lhs_ty != return_ty.as_ref() {
                    //         return Err(format!(
                    //             "Return type of function and variable type differ: expected {:?}; actual: {:?}",
                    //             return_ty, lhs_ty,
                    //         ));
                    //     }
                    // } // Else case does not need to be checked, as possibly discarding the return value is OK

                    for (argument_expr, expected_ty) in es.iter_mut().zip(param_types.iter()) {
                        argument_expr.type_check(context)?;

                        let argument_expr_ty = argument_expr.type_of();

                        if argument_expr_ty != expected_ty {
                            match (&argument_expr_ty.ty, &expected_ty.ty) {
                                (SimpleType::Array(_t1, _), SimpleType::Ptr(_t2)) => {
                                    // TODO.
                                },
                                _ => return Err(format!("Argument type of function and argument differ: expected: {:?}; actual: {:?}", expected_ty, argument_expr_ty))
                            }
                        }
                    }
                } else {
                    return Err(format!(
                        "Function expression calls an expression which is not a function type {:?}",
                        e1,
                    ));
                }
            }
            Expr::EId(id, ty) => {
                let ctx_ty = context
                    .get_ty(id)
                    .ok_or(format!("Identifier {} not found in typing context.", id))?;

                // TODO: is this the right place to interpret this type as a pointer?
                // if let Type::Array(inner_ty, _) = ctx_ty.ty {
                //     ctx_ty = Type::Ptr(inner_ty);
                // }
                // ctx_ty = ctx_ty.parameter_conversion();

                *ty = ctx_ty;
            }
            Expr::EDeref(e, ty) => {
                e.type_check(context)?;

                if let SimpleType::Ptr(inner_ty) = &e.type_of().ty {
                    *ty = *inner_ty.clone();
                } else {
                    return Err(format!("Cannot dereference non-pointer type"));
                }
            }
            Expr::EAddrOf(e, ty) => {
                e.type_check(context)?;
                *ty = Type::new(SimpleType::Ptr(Box::new(e.type_of().clone())));
            }
            Expr::EUnOp(_un_op, e, ty) => {
                e.type_check(context)?;
                // All supported unary operators evaluate to an integer for now.
                // TODO: write out valid operand argument types.
                *ty = Type::new(SimpleType::I32);
            }
            Expr::EBinOp(_bin_op, e1, e2, ty) => {
                e1.type_check(context)?;
                e2.type_check(context)?;

                // TODO: write out valid operand argument types. This is a bit
                // hacky, but will be sufficient for now.
                match (&e1.type_of().ty, &e2.type_of().ty) {
                    (SimpleType::Ptr(inner_ty), SimpleType::I32) => {
                        *ty = Type::new(SimpleType::Ptr(inner_ty.clone()));
                    }
                    (SimpleType::Array(inner_ty, _), SimpleType::I32) => {
                        *ty = Type::new(SimpleType::Ptr(inner_ty.clone()));
                    }
                    (SimpleType::I32, SimpleType::I32) => {
                        *ty = Type::new(SimpleType::I32);
                    }
                    (SimpleType::Ptr(_), SimpleType::Ptr(_)) => {
                        *ty = Type::new(SimpleType::I32);
                    }
                    (_, _) => {
                        return Err(format!(
                            "Unsupported binary operand types {:?}, {:?}.",
                            e1.type_of(),
                            e2.type_of()
                        ));
                    }
                }
            }
        }
        log::debug!("Typed expression '{:?}' as '{:?}'", self, self.type_of());
        Ok(())
    }
}

impl TypeCheck for Statement {
    fn type_check(&mut self, context: &mut TypingContext) -> Result<(), String> {
        match self {
            Statement::SSkip | Statement::SContinue | Statement::SBreak => {}
            Statement::SAssign(e1, e2) => {
                if !e1.is_lvalue() {
                    return Err(format!(
                        "Left-hand side of assignment must be an lvalue expression"
                    ));
                }

                e1.type_check(context)?;
                e2.type_check(context)?;

                if e1.type_of() != e2.type_of() {
                    return Err(format!(
                        "Type error in assignment statement: {:?}, {:?}",
                        e1.type_of(),
                        e2.type_of()
                    ));
                }
            }
            Statement::SCall(e1, es) => {
                e1.type_check(context)?;

                if let SimpleType::Function(ref param_types, _return_ty) = &e1.type_of().ty {
                    // Check return type
                    // if let Some(e1) = e1 {
                    //     e1.type_check(context)?;

                    //     let lhs_ty = e1.type_of();
                    //     if lhs_ty != return_ty.as_ref() {
                    //         return Err(format!(
                    //             "Return type of function and variable type differ: expected {:?}; actual: {:?}",
                    //             return_ty, lhs_ty,
                    //         ));
                    //     }
                    // } // Else case does not need to be checked, as possibly discarding the return value is OK

                    for (argument_expr, expected_ty) in es.iter_mut().zip(param_types.iter()) {
                        argument_expr.type_check(context)?;

                        let argument_expr_ty = argument_expr.type_of();

                        if argument_expr_ty != expected_ty {
                            match (&argument_expr_ty.ty, &expected_ty.ty) {
                                (SimpleType::Array(_t1, _), SimpleType::Ptr(_t2)) => {
                                    // TODO.
                                },
                                _ => return Err(format!("Argument type of function and argument differ: expected: {:?}; actual: {:?}", expected_ty, argument_expr_ty))
                            }
                        }
                    }
                } else {
                    return Err(String::from(
                        "Function statement calls an expression which is not a function type",
                    ));
                }
            }
            Statement::SSequence(s1, s2) => {
                s1.type_check(context)?;
                s2.type_check(context)?;
            }
            Statement::SIfThenElse(e1, s1, s2) => {
                e1.type_check(context)?;

                if e1.type_of().ty != SimpleType::I32 {
                    return Err(format!(
                        "Type of if condition should be int but is {:?} for expr {:?}",
                        e1.type_of(),
                        e1
                    ));
                }

                s1.type_check(context)?;
                s2.type_check(context)?;
            }
            Statement::SWhile(e, body) => {
                e.type_check(context)?;

                if &e.type_of().ty != &SimpleType::I32 {
                    return Err(String::from("Type of While condition should be int"));
                }

                body.type_check(context)?;
            }
            Statement::SFor(s1, e, s2, s) => {
                s1.type_check(context)?;
                e.type_check(context)?;
                if &e.type_of().ty != &SimpleType::I32 {
                    return Err(String::from("Type of For condition should be int"));
                }
                s2.type_check(context)?;
                s.type_check(context)?;
            }
            Statement::SReturn(e) => {
                if let Some(e) = e {
                    e.type_check(context)?;
                }
            }
            Statement::SMalloc(e1, e2) => {
                e1.type_check(context)?;
                e2.type_check(context)?;

                if let &SimpleType::Ptr(_) = &e1.type_of().ty {
                    if &e2.type_of().ty != &SimpleType::I32 {
                        return Err(String::from(
                            "Type of argument to malloc should be an integer",
                        ));
                    }
                } else {
                    return Err(String::from(
                        "Type of lvalue assignment of malloc should be a pointer",
                    ));
                }
            }
            Statement::SFree(e) => {
                e.type_check(context)?;
                // TODO: should be ptr type or more specific?
                if !e.type_of().is_ptr() {
                    return Err(String::from("Type of argument to free should be a pointer"));
                }
            }
            Statement::SPrint(_, args) => {
                for arg in args {
                    arg.type_check(context)?;
                    if &arg.type_of().ty != &SimpleType::I32 {
                        return Err(format!("Don't know how to print a {:?}", arg));
                    }
                }
            }
        }

        Ok(())
    }
}

impl TypeCheck for FunDef {
    fn type_check(&mut self, context: &mut TypingContext) -> Result<(), String> {
        context.local.clear();

        for (id, ty, init_expr) in &mut self.parameters {
            context.local.insert(id.clone(), ty.clone());

            if let Some(e) = init_expr {
                e.type_check(context)?;

                if e.type_of() != ty {
                    return Err(format!("Type of initial expression '{:?}' for variable '{}' is not of the right type.", e, id));
                }
            }
        }
        for (id, ty, init_expr) in &mut self.local_var_decls {
            context.local.insert(id.clone(), ty.clone());

            if let Some(e) = init_expr {
                e.type_check(context)?;

                if e.type_of() != ty {
                    return Err(format!("Type of initial expression '{:?}' for variable '{}' is not of the right type.", e, id));
                }
            }
        }

        self.body.type_check(context)?;

        Ok(())
    }
}

impl TypeCheck for Program {
    fn type_check(&mut self, context: &mut TypingContext) -> Result<(), String> {
        for (id, ty, e) in &mut self.var_decls {
            ty.tag_restrict();
            context.global.insert(id.clone(), ty.clone());

            if let Some(e) = e {
                e.type_check(context)?;
                if e.type_of() != ty {
                    return Err(format!("Type of initial expression '{:?}' for variable '{}' is not of the right type.", e, id));
                }
            }
        }

        for fun_decl in &mut self.fun_decls {
            context
                .functions
                .insert(fun_decl.0.clone(), fun_decl.1.get_type());
        }

        for fun_decl in &mut self.fun_decls {
            fun_decl.1.type_check(context)?;
        }

        Ok(())
    }
}

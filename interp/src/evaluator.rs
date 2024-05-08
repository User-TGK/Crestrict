use crate::ast::*;
use crate::memory::*;
use crate::restrict::{RestrictStack, RestrictState, RestrictStatus};

use itertools::izip;

use core::panic;
use std::collections::{BTreeSet, HashMap};

type Env = HashMap<Ident, (AllocId, Option<ScopeIdentifier>)>;
type FunEnv = HashMap<AllocId, FunDef>;

/// Unique scope.
pub type ScopeIdentifier = i32;

/// The State.
/// For convenience, we also includes the G and E environments in this struct.
/// Secondly, we maintain a scope counter to ensure that only fresh scope identifiers
/// are used.
#[derive(Clone, Debug)]
pub struct State {
    /// Clight global environment (G):
    /// Map from global variables to block references
    /// and map from function references to function definitions
    pub global: (Env, FunEnv),

    /// Clight local environment (E):
    /// Map from local variables to block references
    pub env: Env,

    /// Clight Memory (M):
    /// Map from block references to bounds and contents
    pub memory: Memory,

    /// Restrict context:
    /// Scope stack with restrict information
    pub restrict_state: RestrictStack,

    /// si -> B, tracks scopes which are still alive
    pub scopes: HashMap<i32, bool>,

    /// Generate fresh IDs for block scopes
    scope_counter: i32,
}

impl State {
    /// Return the currently active scope.
    pub fn get_scope(&self) -> ScopeIdentifier {
        self.restrict_state.last().unwrap().get_scope().to_owned()
    }

    /// Get the block from a symbol.
    pub fn symbol(&self, iden: &Ident) -> Option<AllocId> {
        self.global.0.get(iden).copied().map(|(block, _)| block)
    }

    /// Get the function definition from a block.
    pub fn funct(&self, block: &AllocId) -> Option<FunDef> {
        self.global.1.get(block).cloned()
    }

    /// Allocate blocks in the memory for a list of declarations.
    pub fn alloc_vars(&mut self, decls: &Vec<Dcl>) -> Vec<AllocId> {
        let mut blocks = vec![];
        for (id, ty, _) in decls {
            let block = self.memory.alloc(ty.sizeof());
            self.env.insert(id.clone(), (block, Some(self.get_scope())));

            log::debug!(
                "Storing {} at block {} declared in scope {}",
                id,
                block,
                self.get_scope()
            );

            blocks.push(block);
        }

        blocks
    }

    /// Bind the parameter values to their blocks in memory.
    pub fn bind_params(
        &mut self,
        blocks: &[AllocId],
        tys: &[Type],
        vargs: &[Val],
        _scope: &ScopeIdentifier,
    ) -> Result<(), String> {
        for (block, _ty, varg) in izip!(blocks, tys, vargs) {
            self.memory.store(*block, 0, varg.clone())?;
        }
        Ok(())
    }

    pub fn load(&mut self, loc: &mut Loc) -> Result<Val, String> {
        log::debug!("Reading from {:?}", loc);

        self.check_restrict(false, loc)?;

        self.memory.load(loc.block(), loc.offset())
    }

    pub fn store(&mut self, loc: &mut Loc, val: Val) -> Result<(), String> {
        log::debug!("Storing {:?} at {:?}", val, loc);

        self.check_restrict(true, loc)?;

        self.memory.store(loc.block(), loc.offset(), val)
    }

    #[allow(unused_assignments)]
    pub fn check_restrict(&mut self, write: bool, loc: &mut Loc) -> Result<(), String> {
        // Fix 4.4: Filter bases w.r.t. active scopes
        // the place where we perform this filter slightly differs from the
        // operational semantics. This allows the recursive call
        // for 4.2.2 to just pass the unfiltered location to the recursive invocation.
        loc.remove_inactive_bases(&self.scopes);

        let loc_stripped = loc.strip_prov();
        let bases = loc.get_bases();

        // Determine the new restrict state depending on whether we are doing a write or
        // a read access.
        let mut new_restrict_state = match write {
            true => {
                // Fix 4.2.2: modification of the restrict object itself.
                for (base_loc, _) in bases {
                    self.check_restrict(true, &mut base_loc.clone())?;

                    log::debug!(
                        "For {:?}, setting {:?} to modified.",
                        loc_stripped,
                        base_loc
                    );
                }

                // Distinction 4.2.3 omitted.
                RestrictState::Restricted(bases.clone())
            }

            false => {
                let mut bases_fam = BTreeSet::new();

                // Distinction 4.2.3 omitted.
                bases_fam.insert(bases.clone());

                RestrictState::OnlyRead(bases_fam)
            }
        };

        let mut scope_to_filter = None;
        let mut start: bool = true;

        // Fix 4.3: check access with all restrict maps, not only at the top.
        for restrict_status in self.restrict_state.iter_mut().rev() {
            if let Some(scope) = scope_to_filter {
                new_restrict_state = new_restrict_state.filter_bases(scope);
            }

            // If there already was a state for this access, join their states if there
            // is no conflict.
            if let Some(old_restrict_state) = restrict_status.states.get(&loc_stripped) {
                let joined_state = new_restrict_state
                    .clone()
                    .join_restrict_state(old_restrict_state);

                if let RestrictState::Poison = joined_state {
                    log::debug!(
                        "Conflict at {:?}, old: {:?}, new: {:?}",
                        loc,
                        old_restrict_state,
                        new_restrict_state
                    );
                    return Err(String::from("An object which has been modified is accessed through an expression based on a restrict-qualified pointer and another lvalue not also based on said pointer."));
                }

                restrict_status.states.insert(loc_stripped, joined_state);
            }
            // Otherwise, simply store the new state.
            else {
                if start {
                    restrict_status
                        .states
                        .insert(loc_stripped, new_restrict_state.clone());
                }
            }

            scope_to_filter = Some(restrict_status.get_scope());
        }
        start = false;

        log::trace!("{:#?}", self.restrict_state);

        Ok(())
    }

    /// Rule rmerge of the operational semantics.
    ///
    pub fn exit_restrict_block(
        &mut self,
        arg_ids: &Vec<AllocId>,
        local_var_ids: &Vec<AllocId>,
    ) -> Result<(), String> {
        if let Some(restrict_state) = self.restrict_state.pop() {
            // After popping main, only global vars remain but we do not have to do anything.
            if self.restrict_state.is_empty() {
                return Ok(());
            }

            for loc in restrict_state.states.keys() {
                if !arg_ids.contains(&loc.block) && !local_var_ids.contains(&loc.block) {
                    // The modified address was not a local variable, so we need to merge.

                    let scope = restrict_state.get_scope();
                    let state = restrict_state.states[loc].filter_bases(scope);

                    // The restrict state which is now on top of the stack.
                    let last_restrict_map = self.restrict_state.last_mut().unwrap();

                    // We dont have to join: If there was a value it must already have been joined
                    // due to fix 4.3. Only if there is no value yet we propagate the state after filtering.
                    if last_restrict_map.states.get(loc).is_none() {
                        last_restrict_map.states.insert(loc.clone(), state);
                    }
                }
            }
        } else {
            unreachable!("Tried to exit a restrict block but the stack was empty...");
        }

        Ok(())
    }

    pub fn new_scope(&mut self) -> i32 {
        let scope = self.scope_counter;
        self.scope_counter += 1;
        self.scopes.insert(scope, true);

        scope
    }
}

// Statement Outcomes
#[derive(Debug)]
pub enum Outcome {
    Normal,
    Continue,
    Break,
    Return,
    ReturnValue(Val),
}

impl Outcome {
    /// Outcome updates (at the end of a loop execution)
    fn loop_outcome_update(self) -> Option<Self> {
        match &self {
            Outcome::Break => Some(Outcome::Normal),
            Outcome::Return | Outcome::ReturnValue(_) => Some(self),
            _ => None,
        }
    }
}

fn evaluate_un_op_expr(op: &UnOp, expr: &Expr, state: &mut State) -> Result<Val, String> {
    match (op, evaluate_expr(expr, state)?) {
        (UnOp::ONotBool, v) => Ok(Val::from_bool(!v.is_true())),
        (UnOp::ONotInt, Val::I32(v)) => Ok(Val::I32(!(v))),
        (UnOp::ONeg, Val::I32(v)) => Ok(Val::I32(-v)),
        (_, _) => Err(String::from("Unary operator called with invalid argument")),
    }
}

fn evaluate_bin_op_expr(
    op: &BinOp,
    e1: &Expr,
    e2: &Expr,
    state: &mut State,
) -> Result<Val, String> {
    match (op, evaluate_expr(e1, state)?, evaluate_expr(e2, state)?) {
        // Pointer arithmetic (ommitted ++ and -- unary pointer arithmetic operations)
        // a1[a2] ≡ *(a1 + a2)
        (BinOp::OAdd, Val::Ptr(l), Val::I32(rhs)) => Ok(Val::Ptr(l.add(rhs))),
        (BinOp::OAdd, Val::I32(rhs), Val::Ptr(l)) => Ok(Val::Ptr(l.add(rhs))),

        (BinOp::OSub, Val::Ptr(l), Val::I32(rhs)) => Ok(Val::Ptr(l.add(-rhs))),

        (BinOp::OEq, Val::Ptr(l1), Val::Ptr(l2)) => {
            // We treat pointers to the same location but with different provenance equal,
            // although we dont *have* to: https://www.open-std.org/jtc1/sc22/wg14/www/docs/dr_260.htm
            // This does not seem fundamental for the semantics.
            Ok(Val::from_bool(l1.simple() == l2.simple()))
        }

        // Logical / arithmetic expressions
        (BinOp::OAdd, Val::I32(v1), Val::I32(v2)) => Ok(Val::I32(v1 + v2)),
        (BinOp::OSub, Val::I32(v1), Val::I32(v2)) => Ok(Val::I32(v1 - v2)),
        (BinOp::OMul, Val::I32(v1), Val::I32(v2)) => Ok(Val::I32(v1 * v2)),
        (BinOp::ODiv, Val::I32(v1), Val::I32(v2)) => Ok(Val::I32(v1 / v2)),
        (BinOp::OMod, Val::I32(v1), Val::I32(v2)) => Ok(Val::I32(v1 % v2)),
        (BinOp::OAnd, Val::I32(v1), Val::I32(v2)) => Ok(Val::I32(v1 & v2)),
        (BinOp::OOr, Val::I32(v1), Val::I32(v2)) => Ok(Val::I32(v1 | v2)),
        (BinOp::OXor, Val::I32(v1), Val::I32(v2)) => Ok(Val::I32(v1 ^ v2)),
        (BinOp::OShl, Val::I32(v1), Val::I32(v2)) => Ok(Val::I32(v1 << v2)),
        (BinOp::OShr, Val::I32(v1), Val::I32(v2)) => Ok(Val::I32(v1 >> v2)),
        (BinOp::OEq, Val::I32(v1), Val::I32(v2)) => Ok(Val::from_bool(v1 == v2)),
        (BinOp::ONe, Val::I32(v1), Val::I32(v2)) => Ok(Val::from_bool(v1 != v2)),
        (BinOp::OLt, Val::I32(v1), Val::I32(v2)) => Ok(Val::from_bool(v1 < v2)),
        (BinOp::OGt, Val::I32(v1), Val::I32(v2)) => Ok(Val::from_bool(v1 > v2)),
        (BinOp::OLe, Val::I32(v1), Val::I32(v2)) => Ok(Val::from_bool(v1 <= v2)),
        (BinOp::OGe, Val::I32(v1), Val::I32(v2)) => Ok(Val::from_bool(v1 >= v2)),
        (_, _, _) => Err(String::from(
            "Binary operator called with invalid arguments",
        )),
    }
}

fn evaluate_expr(expr: &Expr, state: &mut State) -> Result<Val, String> {
    match expr {
        Expr::EConstInt(i, _) => Ok(Val::I32(*i)),
        Expr::ESizeOf(ty, _) => Ok(Val::I32(ty.sizeof() * 4)),
        Expr::EUnOp(un_op, e, _) => {
            Ok(evaluate_un_op_expr(un_op, e, state).expect("Un op type error"))
        }
        Expr::EBinOp(bin_op, e1, e2, _) => evaluate_bin_op_expr(bin_op, e1, e2, state),
        Expr::EAddrOf(l_value_expr, _) => Ok(Val::Ptr(evaluate_lvalue_expr(l_value_expr, state)?)),
        Expr::ECall(e1, args, _) => evaluate_fun_invocation(e1, args, state),
        _ => {
            let mut loc = evaluate_lvalue_expr(expr, state)?;

            let mut val = match expr.type_of().ty {
                SimpleType::Array(_, _) => Val::Ptr(loc.clone()),
                _ => state.load(&mut loc)?,
            };

            // get_restrict_scope
            if expr.type_of().is_restrict() {
                let scope = match expr.type_of().get_restrict_block() {
                    Some(si) => si,
                    None => state.get_scope(),
                };

                val.add_prov((loc, scope));
            }

            Ok(val)
        }
    }
}

fn evaluate_lvalue_expr(expr: &Expr, state: &mut State) -> Result<Loc, String> {
    match expr {
        Expr::EId(id, _ty) => match state.env.get(id).copied() {
            Some((a, _)) => Ok(Loc::new(a, 0)),
            None => state.symbol(id).map_or(
                Err(String::from("Symbol '") + id + "' missing from environments {G, E}"),
                |block| Ok(Loc::new(block, 0)),
            ),
        },
        Expr::EDeref(ptr_expr, _) => match evaluate_expr(ptr_expr, state)? {
            Val::Ptr(loc) => Ok(loc),
            _ => Err(String::from("Unknown result type for deref expr")),
        },
        _ => panic!(
            "Unexpected Expr type passed to evaluate_lvalue_expr {:?}",
            expr
        ),
    }
}

fn evaluate_fun_invocation(e1: &Expr, args: &Vec<Expr>, state: &mut State) -> Result<Val, String> {
    log::debug!("New function invocation {:?}", e1);

    // Get the block identifier for the invoked function
    let func_ptr = evaluate_lvalue_expr(e1, state)?.block();
    let mut vargs = vec![];

    // Evaluate all arguments to values
    for arg in args {
        vargs.push(evaluate_expr(arg, state)?);
    }

    if let Expr::EId(id, _) = e1 {
        // Skip main, as we require it to be scope 0.
        if id != "main" {
            let scope = state.new_scope();

            // Start a new restrict block.
            state.restrict_state.push(RestrictStatus::new(scope));
        }
    }

    let scope_id = state.get_scope();

    // Retrieve the function definition fd of the function
    let fd = state
        .funct(&func_ptr)
        .ok_or(format!("Undefined function '{:?}'", e1))?;

    // Make a copy of the previous local environment ('previous stack frame')
    let prev_env = state.env.clone();
    state.env.clear();

    // Allocate all variables (local and parameters) and bind the parameter values
    let arg_ids = state.alloc_vars(&fd.parameters);
    let local_var_ids = state.alloc_vars(&fd.local_var_decls);
    state.bind_params(
        &arg_ids,
        &fd.parameters
            .iter()
            .map(|p| p.1.clone())
            .collect::<Vec<Type>>(),
        &vargs,
        &state.get_scope(),
    )?;

    // Set the initial values for local variables if they are provided
    // We have to do this after the calls to alloc_vars, as the variable values
    // may be based on the parameters.
    for (block, (_id, _ty, expr)) in local_var_ids.iter().zip(fd.local_var_decls.iter()) {
        if let Some(init_expr) = expr.as_ref() {
            let init_val: Val = evaluate_expr(init_expr, state)?;

            state.memory.store(*block, 0, init_val)?;
        }
    }

    let out = evaluate_statement(&fd.body, state)?;

    // exit restrict block
    state.exit_restrict_block(&arg_ids, &local_var_ids)?;

    // Free all allocated blocks for this function
    for id in arg_ids {
        state.memory.free(id);
    }
    for id in local_var_ids {
        state.memory.free(id);
    }

    // Restore the stack frame
    state.env = prev_env;

    // Disable the scope
    if let Some(scope_state) = state.scopes.get_mut(&scope_id) {
        *scope_state = false;
    } else {
        panic!("Scope {} missing from scopes map", state.get_scope());
    }

    log::debug!("Finished function invocation {:?}", e1);

    // Check compatibility between return value, outcome and return type
    match (&out, fd.return_type.ty.clone()) {
        (Outcome::Normal, SimpleType::Void) => Ok(Val::Undef),
        (Outcome::Return, SimpleType::Void) => Ok(Val::Undef),
        (Outcome::ReturnValue(_), SimpleType::Void) => Err(String::from(
            "Return value found for Void return type in function",
        )),
        (Outcome::ReturnValue(v), _) => Ok(v.to_owned()),
        (_, _) => unimplemented!(
            "Incompatible return value/outcome/return type: {:?}/{:?}",
            out,
            fd.return_type
        ),
    }
}

fn evaluate_statement(statement: &Statement, state: &mut State) -> Result<Outcome, String> {
    match statement {
        Statement::SSkip => Ok(Outcome::Normal),
        Statement::SBreak => Ok(Outcome::Break),
        Statement::SContinue => Ok(Outcome::Continue),
        Statement::SReturn(e) => match e {
            Some(e) => Ok(Outcome::ReturnValue(evaluate_expr(e, state)?)),
            None => Ok(Outcome::Return),
        },
        Statement::SAssign(e1, e2) => {
            let mut loc = evaluate_lvalue_expr(e1, state)?;
            let val = evaluate_expr(e2, state)?;

            // Note, we assume the type of e1 = type of e2.
            state.store(&mut loc, val)?;
            Ok(Outcome::Normal)
        }
        // Function calls except to printf, malloc and free
        Statement::SCall(e1, args) => {
            let _res: Val = evaluate_fun_invocation(e1, args, state)?;

            Ok(Outcome::Normal)
        }
        Statement::SSequence(s1, s2) => {
            // Note, Clight concats output traces for sequencing.
            let out = evaluate_statement(s1, state)?;
            match out {
                Outcome::Normal => evaluate_statement(s2, state),
                _ => Ok(out),
            }
        }
        Statement::SIfThenElse(expr, true_branch, false_branch) => {
            match evaluate_expr(expr, state)?.is_true() {
                true => evaluate_statement(true_branch, state),
                false => evaluate_statement(false_branch, state),
            }
        }
        Statement::SWhile(expr, inner_statement) => match evaluate_expr(expr, state)?.is_true() {
            true => {
                let out: Outcome = evaluate_statement(inner_statement, state)?;
                match out {
                    Outcome::Normal | Outcome::Continue => evaluate_statement(statement, state),
                    _ => Ok(out),
                }
            }
            false => Ok(Outcome::Normal),
        },
        Statement::SFor(s1, e, s2, s) => {
            if let Outcome::Normal = evaluate_statement(s1, state)? {
                match evaluate_expr(e, state)?.is_true() {
                    true => {
                        // Execute the loop body
                        let out = evaluate_statement(s, state)?;

                        match out {
                            Outcome::Normal | Outcome::Continue => {
                                if let Outcome::Normal = evaluate_statement(s2, state)? {
                                    let statement = Statement::SFor(
                                        Box::new(Statement::SSkip),
                                        e.clone(),
                                        s2.clone(),
                                        s.clone(),
                                    );
                                    evaluate_statement(&statement, state)
                                } else {
                                    Err(String::from(
                                        "Unexpected outcome in executing s2 of for loop",
                                    ))
                                }
                            }
                            _ => {
                                // If this returns an error, it's a bug.
                                out.loop_outcome_update().ok_or(String::from(
                                    "[BUG] Unexpected outcome of for loop body",
                                ))
                            }
                        }
                    }
                    false => Ok(Outcome::Normal),
                }
            } else {
                Err(String::from(
                    "Initial statement in for loop didn't have a Normal outcome",
                ))
            }
        }
        Statement::SMalloc(result_expr, size_expr) => {
            let mut loc = evaluate_lvalue_expr(result_expr, state)?;

            if let Val::I32(size) = evaluate_expr(size_expr, state)? {
                let size = size / 4;

                let val = Loc::new(state.memory.malloc(size), 0);

                state.store(&mut loc, Val::Ptr(val))?;

                Ok(Outcome::Normal)
            } else {
                // Should not be reachable if type checker is OK
                Err(String::from("Malloc argument was not of type I32"))
            }
        }
        Statement::SFree(expr) => {
            if let Val::Ptr(mut loc) = evaluate_expr(expr, state)? {
                state.check_restrict(true, &mut loc)?;
                state.memory.manual_free(loc.block())?;

                Ok(Outcome::Normal)
            } else {
                Err(String::from(
                    "Attempted to free a non-pointer expression. This likely means there is a bug in the type checker.",
                ))
            }
        }
        // Approximation of C printf. For now, only print decimal (%d) arguments.
        Statement::SPrint(format_string, args) => {
            let mut format_string = format_string
                .to_owned()
                .replace("\\n", "\n")
                .replace("\"", "");

            for arg in args {
                if let Val::I32(i) = evaluate_expr(arg, state)? {
                    format_string = format_string.replacen("%d", &i.to_string(), 1);
                } else {
                    return Err(String::from("Attempted to print a non-integer type"));
                }
            }

            // Let's print.
            print!("{}", format_string);
            Ok(Outcome::Normal)
        }
    }
}

pub fn evaluate_program(program: &mut Program) -> Result<Val, String> {
    let mut state = State {
        global: (HashMap::new(), HashMap::new()),
        env: HashMap::new(),
        memory: Memory::new(),
        restrict_state: vec![RestrictStatus::new(0)],
        scope_counter: 1,
        scopes: HashMap::from([(0, true)]),
    };

    // initmem
    // Allocate memory for global variables
    for (var, ty, expr) in &mut program.var_decls {
        let block = state.memory.alloc(ty.sizeof());
        log::debug!("Storing {} at block {}", var, block);
        state.global.0.insert(var.to_string(), (block, Some(0)));

        if let Some(init_expr) = expr.as_ref() {
            let init_val = evaluate_expr(init_expr, &mut state)?;

            state.memory.store(block, 0, init_val)?;
        }
    }

    for (id, fun_def) in &program.fun_decls {
        let block = state.memory.alloc(1);
        log::debug!("Storing {} at block {}", id, block);
        state.global.0.insert(id.to_string(), (block, None));
        state.global.1.insert(block, fun_def.clone());
    }

    let res: Result<Val, String> = evaluate_fun_invocation(
        &Expr::EId(
            program.entry_point.clone(),
            Type::new(SimpleType::Function(
                vec![],
                Box::new(Type::new(SimpleType::I32)),
            )),
        ),
        &vec![],
        &mut state,
    );

    // Deallocate global variables and functions
    for (_, (b, _)) in state.global.0 {
        state.memory.free(b);
    }

    res
}

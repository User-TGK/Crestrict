/// A Clight inspired subset of the C11 programming language.
///
/// The intention of the language provided in this module is to
/// capture all relevant C11 language constructs to give a useful
/// semantics for the restrict type qualifier keyword. This includes
/// but is not limited to: pointer arithmetics, arrays, malloc/free,
/// functions, global variables.
///
use crate::evaluator::ScopeIdentifier;
use crate::memory::Loc;
use crate::restrict::Base;

/// Identifiers are UTF Rust strings.
pub type Ident = String;

/// A declaration is a combination of an identifier, a type and an optional
/// initial value.
///
/// Example:
///
/// int x;       // Integer variable without initial value
/// int y = 0;   // Integer variable with initial value 0
/// int* z = &y; // Pointer to integer variable with initial value y's address
///
pub type Dcl = (Ident, Type, Option<Expr>);

/// A function declaration is a combination of an identifier and a function
/// definition.
///
/// Example:
///
/// void foo(int x); // Function that takes a single integer as argument and returns nothing
///
pub type FunDcl = (Ident, FunDef);

/// (C11 6.7.3) Type Qualifiers
#[derive(Clone, Debug, PartialEq)]
pub enum TypeQualifier {
    Restrict,
}

#[derive(Clone, Debug)]
pub enum TypeModifier {
    RestrictScope(ScopeIdentifier),
}

#[derive(Clone, Debug)]
pub struct Type {
    pub ty: SimpleType,
    pub qualifiers: Vec<TypeQualifier>,
    pub modifiers: Vec<TypeModifier>,
}

/// (C11 6.7.2) Type (Specifiers)
#[derive(Clone, Debug, PartialEq)]
pub enum SimpleType {
    /// General integer type to capture all integer values
    I32,

    /// Pointer type
    /// Component 0 is the inner type
    /// Component 1 is a list of type qualifiers
    Ptr(Box<Type>),

    /// Array type
    /// Component 0 is the inner type
    /// Component 1 is the length of the array
    Array(Box<Type>, i32),

    /// Function type
    /// Component 0 is an ordered list of the parameter type
    /// Component 1 is the return type
    /// Note: we omitted CompCert component 3: calling convention
    Function(Vec<Type>, Box<Type>),

    /// Void type
    /// Only valid as function return type
    Void,

    /// Undefined type
    /// The None type with which the expressions will be annotated
    /// before type checking
    Undef,
}

impl Type {
    pub fn parameter_conversion(&self) -> Self {
        let ty = match self.ty.clone() {
            SimpleType::Array(t, _) => SimpleType::Ptr(Box::new(t.parameter_conversion())),
            SimpleType::Ptr(t) => SimpleType::Ptr(Box::new(t.parameter_conversion())),
            SimpleType::Function(tys, return_ty) => SimpleType::Function(
                tys.into_iter()
                    .map(|ty| ty.parameter_conversion())
                    .collect(),
                Box::new(return_ty.parameter_conversion()),
            ),
            _ => self.ty.clone(),
        };

        Type {
            ty,
            qualifiers: self.qualifiers.clone(),
            modifiers: self.modifiers.clone(),
        }
    }

    pub fn new(simple_type: SimpleType) -> Self {
        Self {
            ty: simple_type,
            qualifiers: vec![],
            modifiers: vec![],
        }
    }

    pub fn with_qualifiers(mut self, qualifiers: Vec<TypeQualifier>) -> Self {
        self.qualifiers = qualifiers;

        self
    }

    pub fn tag_restrict(&mut self) {
        if self.is_restrict() {
            self.modifiers.push(TypeModifier::RestrictScope(0));
        }

        match &mut self.ty {
            SimpleType::Array(inner_ty, _) => inner_ty.tag_restrict(),
            SimpleType::Ptr(inner_ty) => inner_ty.tag_restrict(),
            _ => {}
        }
    }

    /// How many memory cells does a type occupy.
    pub fn sizeof(&self) -> i32 {
        match &self.ty {
            SimpleType::I32
            | SimpleType::Ptr(_)
            | SimpleType::Function(_, _)
            | SimpleType::Void => 1,
            SimpleType::Array(ty, size) => size * ty.sizeof(),
            SimpleType::Undef => unimplemented!(
                "sizeof() a type may only be used after annotating expressions with their types"
            ),
        }
    }

    /// Whether the type is a pointer.
    pub fn is_ptr(&self) -> bool {
        match self.ty {
            SimpleType::Ptr(_) => true,
            _ => false,
        }
    }

    /// Whether the type is restrict qualified.
    pub fn is_restrict(&self) -> bool {
        self.qualifiers.contains(&TypeQualifier::Restrict)
    }

    pub fn get_restrict_block(&self) -> Option<i32> {
        for modifier in &self.modifiers {
            let TypeModifier::RestrictScope(scope_id) = modifier;
            return Some(*scope_id);
        }

        None
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (&self.ty, &other.ty) {
            (SimpleType::I32, SimpleType::I32) | (SimpleType::Void, SimpleType::Void) => true,
            (SimpleType::Ptr(inner_ty_1), SimpleType::Ptr(inner_ty_2)) => inner_ty_1 == inner_ty_2,
            (SimpleType::Array(inner_ty_1, size_1), SimpleType::Array(inner_ty_2, size_2)) => {
                (inner_ty_1 == inner_ty_2) && (size_1 == size_2)
            }
            (SimpleType::Array(inner_ty_1, _), SimpleType::Ptr(inner_ty_2)) => {
                inner_ty_1 == inner_ty_2
            }
            (SimpleType::Ptr(inner_ty_1), SimpleType::Array(inner_ty_2, _)) => {
                inner_ty_1 == inner_ty_2
            }
            (
                SimpleType::Function(param_tys_1, return_ty_1),
                SimpleType::Function(param_tys_2, return_ty_2),
            ) => (param_tys_1 == param_tys_2) && (return_ty_1 == return_ty_2),
            (_, _) => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum UnOp {
    ONotBool,
    ONotInt,
    ONeg,
}

#[derive(Clone, Debug)]
pub enum BinOp {
    OAdd,
    OSub,
    OMul,
    ODiv,
    OMod,
    OAnd,
    OOr,
    OXor,
    OShl,
    OShr,
    OEq,
    ONe,
    OLt,
    OGt,
    OLe,
    OGe,
}

#[derive(Clone, Debug)]
pub enum Expr {
    EConstInt(i32, Type),
    EId(Ident, Type),
    ECall(Box<Expr>, Vec<Expr>, Type), // Allow function calls as expressions
    EDeref(Box<Expr>, Type),
    EAddrOf(Box<Expr>, Type),
    EUnOp(UnOp, Box<Expr>, Type),
    EBinOp(BinOp, Box<Expr>, Box<Expr>, Type),
    ESizeOf(Type, Type),
}

impl Expr {
    pub fn type_of(&self) -> &Type {
        match self {
            Expr::ECall(_, _, ty) => ty,
            Expr::EConstInt(_, ty) => ty,
            Expr::EId(_, ty) => ty,
            Expr::EDeref(_, ty) => ty,
            Expr::EAddrOf(_, ty) => ty,
            Expr::EUnOp(_, _, ty) => ty,
            Expr::EBinOp(_, _, _, ty) => ty,
            Expr::ESizeOf(_, ty) => ty,
        }
    }

    pub fn is_lvalue(&self) -> bool {
        match self {
            Expr::EId(_, _) | Expr::EDeref(_, _) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Val {
    I32(i32),
    Ptr(Loc),
    Undef,
}

impl Val {
    pub fn is_true(&self) -> bool {
        match self {
            Val::Undef => false,
            _ => !self.is_false(),
        }
    }

    pub fn is_false(&self) -> bool {
        match self {
            Val::I32(0) => true,
            _ => false,
        }
    }

    /// C does not have a boolean type.
    /// Although any type may represent a boolean value in C11, we only allow integers.
    pub fn from_bool(b: bool) -> Self {
        match b {
            true => Val::I32(1),
            false => Val::I32(0),
        }
    }

    pub fn add_prov(&mut self, base: Base) {
        if let Self::Ptr(l) = self {
            l.add_prov(base);
        }
    }
}

#[derive(Clone, Debug)]
pub enum Statement {
    SSkip,
    SAssign(Expr, Expr),
    SSequence(Box<Statement>, Box<Statement>),
    SIfThenElse(Expr, Box<Statement>, Box<Statement>),
    SWhile(Expr, Box<Statement>),
    SFor(Box<Statement>, Expr, Box<Statement>, Box<Statement>),
    SBreak,
    SContinue,
    SReturn(Option<Expr>),

    SCall(Expr, Vec<Expr>), // A limited variant of 'expressions as statements'
    SMalloc(Expr, Expr),    // Represents C's void* malloc(size_t size)
    SFree(Expr),            // Represents C's void free(void* ptr)
    SPrint(String, Vec<Expr>), // Represents C's printf. A bit hacky as we don't actually have string/char[] as a type.
}

#[derive(Clone, Debug)]
pub struct FunDef {
    pub return_type: Type,
    pub parameters: Vec<Dcl>,
    pub local_var_decls: Vec<Dcl>,
    pub body: Statement,
}

impl FunDef {
    pub fn get_type(&self) -> Type {
        let param_types: Vec<Type> = self.parameters.iter().map(|p| p.1.clone()).collect();

        Type::new(SimpleType::Function(
            param_types,
            Box::new(self.return_type.clone()),
        ))
    }
}

// A Clight-like program
#[derive(Debug)]
pub struct Program {
    pub var_decls: Vec<Dcl>,
    pub fun_decls: Vec<FunDcl>,
    pub entry_point: Ident,
}

impl Default for Program {
    fn default() -> Self {
        Program {
            var_decls: Vec::new(),
            fun_decls: Vec::new(),
            entry_point: String::from("main"),
        }
    }
}

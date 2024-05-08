KCC generally give UB for main1 and main3:

// C-in-K semantics do give UB here, opposite to `indirection-aliasing.c`.
// xq is in a different location than xp obviously, so apparently
// this is sufficient to detect UB for C-in-K.

// Which rewrite rule treats provenance for nested locations?
// Call to `instantiate` upon lvalue conversion:
// https://github.com/kframework/c-semantics/blob/master/semantics/c/language/common/expr/eval.k#L58
//
// Rule for `instantiating` restrict qualified type T:
// https://github.com/kframework/c-semantics/blob/master/semantics/c/language/execution/expr/identifier.k#L24 
//
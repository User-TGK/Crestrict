// This example is meant to clarify on why we use `SimpleLoc` in the bases for `RestrictState` items,
// opposed to using `Loc` as we do for bases in pointer values.

// The main takeaways are that:
//
// 1. In pointer values we want to be able to backtrack all locations, so that we can respect the
//    "Every access that modifies X shall be considered also to modify P" clause.
// 2. In the restrict state, we are only concerned about the 'direct' bases to determine whether
//    the access is valid or not.

// First, observe that this distinction is only relevant for multiple levels of restrict (i.e. the base of 
// a location must have a base itself).
//
// It means that we are in the following situation:
//
// p -> r -> X
// q -> r -> X
//
// [Pointer values]
// lvalue **p:      (slx, {( (slr, { ( ( slp, {} ),  si ) }), si )})
// lvalue **q:      (slx, {( (slr, { ( ( slq, {} ),  si ) }), si )})
//
// [RestrictState values]
// **p: ( slx, {( slr,  si )} )
// **q: ( slx, {( slr,  si )} )
//
//
//          SlBase                      Base
// **p      {( slr,  si )}              {( (slr, { ( ( slp, {} ),  si ) }), si )}
// **q      {( slr,  si )}              {( (slr, { ( ( slq, {} ),  si ) }), si )}
//
// So for **p and **q we have that:
// SlBase(**p) == SlBase(**q)   AND
//   Base(**p) !=   Base(**q)
//

int foo(int *restrict *restrict p, int *restrict *restrict q) {
    **p = 10;
    **q = 11;
    return **p;
}

int main() {
    int X;
    int* r = &X;

    foo(&r, &r);

    return 0;
}

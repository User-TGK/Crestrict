// RestrictCheck: DB
// KCC: UB
// Standard: DB

/**
 * EXAMPLE 3 (Source: C99 standard)
 *
 * ```
 * The function parameter declarations illustrate how an unmodified object can be aliased through two restricted pointers.
 * In particular, if a and b are disjoint arrays, a call of the form h(1, a, b, b) has defined behavior,
 * because array b is not modified within function h.
 * ```
 */

#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

void h(int n, int *restrict p, int *restrict q, int *restrict r)    // p has base l1, q l2, r l3;
{
    for (int i = 0; i < n; i++)
        p[i] = q[i] + r[i];                     // load q[0]:   restrict(l) = OnlyRead({(l2, functionScope(1))})
                                                // load q[1]: join the current state with OnlyRead({(l3, functionScope(1))})
                                                // =>           restrict(l) = Unrestricted

    // Now, since l is not a local variable nor parameter but is in the restrict of this block,
    // we need to merge with the state from the last restrict block
    // But Unrestricted and Restricted({}) don't merge merge according to the model.
    // So this would give UB (but it doesn't have UB).
}

int main()
{
    // needed to trigger kcc to do restict checks for this block.
    int* restrict q;

    int a;
    int b;

    b = 0;

    // Defined behavior because b is not modified.
    h(1, &a, &b, &b);

    printf("%d\n", a);

    return 0;
}

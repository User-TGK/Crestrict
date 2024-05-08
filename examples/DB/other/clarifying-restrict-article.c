/**
 * EXAMPLE 1 (Source: "Clarifying the restrict Keyword", Johnson & Homer)
 * https://www.open-std.org/jtc1/sc22/wg14/www/docs/n2237.pdf
 * 
 * Argues why the following functions do not provide identical information to a translator:
 *
 * void f1(int *p, int *q) { ... }
 * void f2(int * restrict p, int * restrict q) { ... }
 * void f3(int * restrict p, int *q) { ... }
 * void f4(int *p, int * restrict q) { ... }
 *
 * The program below does NOT contain UB (instance of f3). Note that an optimizer may not
 * invalidly optimize `foo` to return 3 (which was a bug in GCC 7.3.0).
 */

#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

void g(int **a, int *b)
{
    *a = b;
}

int foo(int *restrict p, int *q)
{
    g(&q, p); // effectively q = p
    *p = 1;
    *q = 2;

    return *p + *q;
}

int main()
{
    int x;
    int y;

    int z;
    z = foo(&x, &y);

    printf("%d\n", z);
    return 0;
}

/**
 * Source: Defect With Wording Of restrict Specification, Tong et al.
 * https://www.open-std.org/jtc1/sc22/wg14/www/docs/n3025.htm
 *
 * According to the formal definition, and depending whether lines 19-22 are optimizated before
 * the "based on" analysis, a compiler may not be able to optimize line 27.
 */

#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

int f(int* restrict p, int* q)
{
    int *r = p;     // sequence point: r is based on p

    if (r == p)
    {
        r = q;      // optimizing line 17 to int* r = q; means it is no longer based on p.
    }

    *r = 13; // here let E denote &*r == r
    *p = 42;

    return *r;
}

int main()
{
    int x = 1;
    int y = 2;

    int z;
    z = f(&x, &y);

    printf("%d\n", z);

    return 0;
}

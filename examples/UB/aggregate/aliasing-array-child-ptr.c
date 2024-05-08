#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

int foo(int* restrict q[2]) {
    int* r = q[0];
    *(q[1]) = 10;
    *r = 11;
    return *(q[1]);
}

int main() {
    int v[2];                   // Stored at slv

    int* restrict p = v;        // M(p) = ptr((b_v, 0), {(lp, si_{main}}))
    p[0] = 0;
    p[1] = 1;

    int* pa[2];
    pa[0] = &p[0];
    pa[1] = &p[0];

    int x;
    x = foo(pa);

    printf("%d", *p);
    *p = 2;
    printf("%d\n", *p);

    return 0;
}

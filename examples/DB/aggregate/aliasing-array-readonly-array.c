#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

int foo(int* restrict q[2]) {
    return *(q[0]) + *(q[1] - 1);
}

int main() {
    // int v[2] = {0, 1};
    int v[2];
    // v[0] = 0;
    // v[1] = 1;

    int* restrict p = v; // M(p) = ptr((b_v, 0), {b_p, si_{main}})
    p[0] = 0;
    p[1] = 1;

    // int x = foo((int* restrict [2]){&p[0], &p[1]});
    int* pa[2];
    pa[0] = &p[0];
    pa[1] = &p[1];
    int x;
    x = foo(pa);

    printf("%d", *p);
    *p = 2;         // DB: This is a slightly more complex variant of
                    // example 3 of the standard. The array is not modified within foo,
                    // and all loads are done via pointers based on p.
    printf("%d\n", *p);

    return 0;
}

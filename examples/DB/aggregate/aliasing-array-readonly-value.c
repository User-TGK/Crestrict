#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

int foo(int* restrict q[2]) {
    return *(q[0]) + *(q[1]);
}

int main() {
    int v = 0;
    int result;

    int* restrict p[2];
    p[0] = &v;  // M((bp, 0)) = ptr(slv, {(((bp, 0), {}), simain)})
    p[1] = &v;  // M((bp, 1)) = ptr(slv, {(((bp, 1), {}), simain)})

    return foo(p);
}

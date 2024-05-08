#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

void g(int* restrict q1, int* restrict q2) {
    *q1 = 0;
    *q2 = 1; // UB
}

void f (int * restrict p) {
    g(p, p);
}

int main () {
    int x = 0;
    f(&x);

    return 0;
}

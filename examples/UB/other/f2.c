#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

void assign(int* restrict r, int* s) {
    *r = *s;
}

void f(int* restrict p, int* restrict q, int a, int b) {
    for (int i = 0; i < b; ++i) {
        assign(p + i + a, q + i);
    }
}

void init(int* restrict p, int n) {
    for (int i1 = 0; i1 < n; ++i1)
    {
        p[i1] = i1;
    }
}

int main() {
    int d[3];

    init(d, 3);

    f(d, d, 1, 2);

    for (int i2 = 0; i2 < 3; ++i2)
    {
        printf("%d", d[i2]);
    }
    printf("\n");

    return 0;
}
#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

void f(int * restrict p, int* restrict q, int a, int b) {
    for (int i = 0; i < b; ++i) {
        p[i+a] = q[i];
    }
}

int main() {
    int d[50];

    for (int i1 = 0; i1 < 50; ++i1)
    {
        d[i1] = i1; // Write via shared. [Shared]
    }

    f(d, d, 24, 25);

    for (int i2 = 0; i2 < 50; ++i2)
    {
        printf("%d", d[i2]);
    }
    printf("\n");

    return 0;
}
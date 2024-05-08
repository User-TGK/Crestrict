#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

void f1(int * p, int *restrict q, int n)
{
    for (int i = 0; i < n; ++i)
    {
        p[i] = q[i];
    }

    // I assume here that a modification via p[i] of d is a case that is captured by
    // "modifying X (by any means)", because clang gives the wrong output. Still we may need to check
    // this, as p is not based on q. 
}

int main()
{
    int d[50];

    for (int i1 = 0; i1 < 50; ++i1) {
        d[i1] = i1;
    }

    f1(d + 1, d, 25);

    for (int i2 = 0; i2 < 26; ++i2) {
        printf("%d", d[i2]);
    }
    printf("\n");


    return 0;
}

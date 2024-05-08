#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

void f1(int *restrict p, int * q, int n)
{
    for (int i = 0; i < n; ++i)
    {
        p[i] = q[i];
    }
}

int main()
{
    int d[10];

    for (int i1 = 0; i1 < 10; ++i1)
    {
        d[i1] = i1;
    }

    f1(d + 5, d, 5);

    for (int i2 = 0; i2 < 10; ++i2)
    {
        printf("%d", d[i2]);
    }

    printf("\n");

    return 0;
}

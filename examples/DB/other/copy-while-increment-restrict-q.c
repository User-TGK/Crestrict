#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

void f3(int * p, int *restrict q, int n)
{
    while (n > 0)
    {
        *p = *q;
        n = n - 1;
        p = p + 1;
        q = q + 1;
    }
}

int main()
{
    int d[10];

    for (int i1 = 0; i1 < 10; ++i1)
    {
        d[i1] = i1;
    }

    f3(d + 5, d, 5);

    for (int i2 = 0; i2 < 10; ++i2)
    {
        printf("%d", d[i2]);
    }

    printf("\n");

    return 0;
}

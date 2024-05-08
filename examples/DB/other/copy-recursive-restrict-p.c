#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

void f2(int *restrict p, int *q, int n)
{
    if (n <= 0)
    {
        return;
    }

    f2(p + 1, q + 1, n - 1);
    *p = *q;
}

int main()
{
    int d[10];

    for (int i1 = 0; i1 < 10; ++i1)
    {
        d[i1] = i1;
    }

    f2(d + 5, d, 5);

    for (int i2 = 0; i2 < 10; ++i2)
    {
        printf("%d", d[i2]);
    }

    printf("\n");

    return 0;
}

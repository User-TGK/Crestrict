#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

void f1(int *restrict p, int *restrict q, int n)
{
    for (int i = 0; i < n; ++i)
    {
        p[i] = q[i];
    }
}

int main()
{
    int d[50];

    for (int i1 = 0; i1 < 50; ++i1)
    {
        d[i1] = i1;
    }

    f1(d + 1, d, 25);

    for (int i2 = 0; i2 < 26; ++i2)
    {
        printf("%d", d[i2]);
    }
    printf("\n");

    return 0;
}

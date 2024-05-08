#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

void assign(int* restrict r, int* s) {
    *r = *s;
}

void f(int* restrict p, int* restrict q, int a, int n) {
    
    p[a] = 5; 
    
    for (int i = 0; i < n; ++i) {
        assign(p + i + a, q + i);
    }

    int x = p[a];                    
}


int main() {
    int d[50];

    for (int i1 = 0; i1 < 50; ++i1)
    {
        d[i1] = i1;
    }

    int* restrict o = d;
    f(o, o, 0, 25);

    for (int i2 = 0; i2 < 50; ++i2)
    {
        printf("%d", d[i2]);
    }
    printf("\n");

    return 0;
}

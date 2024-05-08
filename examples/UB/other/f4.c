#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

int main()
{
    int u = 0;

    int* p = &u;

    int* restrict *restrict q = &p;     // q -> [p] -> u 
    int z;

    **q = 5;
    *p = 6;

    z = **q;
    printf("%d\n", z);
    
    // int *restrict v = &u;
    // int *restrict * w = &v;     // w is based on v

    // // NOT based on w.
    // int *x = *w;

    return 0;
}

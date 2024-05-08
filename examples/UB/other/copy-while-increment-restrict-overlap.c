/**
 * EXAMPLE 2 (Source: C99 standard)
 * 
 * ```
 * For example, the call to f in main has undefined behavior
 * because each of d[1] through d[49] is accessed through both p and q. ```
 */

#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

void f(int n, int* restrict p, int* restrict q)
{
    while(n > 0) {
        *p = *q;
        p = p+1;
        q = q+1;
        n = n - 1;
    }
}

int main() {
    int d[100];
    for(int i = 0; i < 100; ++i) {
        d[i] = i;
    }

    f(50, d+1, d);

    for(int i = 0; i < 50; ++i) {
        printf("%d", d[i]);
    }
    printf("\n");
}
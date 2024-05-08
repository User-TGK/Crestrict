#include "../../config.h"

#ifdef WITH_STD
#include <stdio.h>
#endif

int foo(int* restrict p, int* q) {  // p in block 3, q in block 4
    // q = p;


    //*p = Loc(2, 0, (3, scope));
    // q = p; would also fix this. // is this true?
    // p = Loc(3,0).
    // SO: it should be added to the value!

    int y;      // stored in block 5

    *p = 5;     // mem access to block 3

    y = *q;

                // printf("%d\n", y);

    return 0;
}

int main() {
    int x = 1;      // stored in block 2.
    foo(&x, &x);    

    return 0;
}
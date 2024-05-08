#include "../../config.h"

#ifdef WITH_STD
#include <stdlib.h>
#endif


void foo(int* r) {
    // *r; // Uncommenting this line would give UB according to C-in-K, while I think that leaving it commented is also UB.
    free(r);
}

int main() {
    int* p;
    int* restrict q;
    p = malloc(sizeof(int));
    q = p;

    *q = 5;
    foo(p);

    return 0;
}

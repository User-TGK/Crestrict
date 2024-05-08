#include "../../config.h"

#ifdef WITH_STD
#include <stdlib.h>
#endif

// 3.3.6 (FP) Call to free

void bar(int* r) {
    free(r);
}

// Both q and r point to the dynamically allocated object, but are not based
// on the same restrict qualified object.
void foo(int* restrict q, int* r) {
    *q = 5;
    bar(r); // Swapping line 8 and 9 would make this program store-after-free,
            // but we want to detect the UB imposed by the restrict rules.
}

int main() {
    int* p;
    p = malloc(sizeof(int));
    foo(p, p);

    return 0;
}

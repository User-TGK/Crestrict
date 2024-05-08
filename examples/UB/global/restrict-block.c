/** Check if the RestrictBlock tags are correctly assign to bases for globals. */

/**
 * Note: if the assignment to p with &x would have scope 1 as base, it would be filtered out when
 * joining restrict states with scope 0. So this shows that the declaration scope is crucial for correctness.
*/

int *restrict p; // Global restrict pointer, but not yet assigned to. Declared in si 0.
int x;

// Scope 1
void foo()
{
    p = &x; // At this point the provenance is assigned.
    *p = 5;
}

// Scope 0
int main()
{
    int *q = &x;
    *q = 6;

    foo();

    return 0;
}

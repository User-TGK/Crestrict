/// UB by overlapping array and "based on" is None

// Copies n integers from q to p
void f (int* restrict p, int* restrict q, int n, int* r) {
    {
        for (int i = 0; i < n; ++i) {
            p[i] = q[i];
        }

        r[n-1] = 0;
    }
}

int main() {
    int d[10];

    for (int i = 0; i < 10; ++i) {
        d[i] = i;
    }

    // UB: in the scope of the invocation of f we access
    // each of d[1] through d[4] via both p and q, in which 
    // the values are also modified (via assignment to p[i]).
    f(d + 1, d, 5, d);

    // Here we create two restrict pointers, d + 1 and d. Although we derive
    // them from d, d is not a restrict pointer so their 'based on' property
    // is None.

    return 0;
}

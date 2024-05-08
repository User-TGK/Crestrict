int x;
int* restrict p;

int foo(int* restrict q) {
    p = q; // p is based on q, but q not on p. This is already UB due to the assignments between restrict rule.

    *q = 10;
    *p = 11;
    return *q;
}

int main() {
    int* restrict _;
    int z;

    z = foo(&x);
    return 0;
}

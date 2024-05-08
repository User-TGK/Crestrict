int x;
int* restrict p = &x;

int foo(int* restrict q, int* restrict r) {
    return *q + *r;
}

int main() {
    int* restrict _;

    *p = 0;
    foo(p, p);

    return 0;
}

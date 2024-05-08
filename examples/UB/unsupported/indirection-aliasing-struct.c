struct bar {
    int *restrict *restrict p;
    int *restrict *restrict q;
};

int foo(struct bar b) {
    **b.p = 10;
    **b.q = 11;
    return **b.p;
}

int main() {
    int* restrict _;

    int x = 0;
    int* xp = &x;

    struct bar b = {&xp, &xp};
    foo(b);

    return 0;
}
int x;
int* restrict p;

int* foo(int* q) {
    return q;
}

int main() {
    int* restrict _;

    p = &x;

    *p = 10;
    p = foo(&x);
    *p = 11;
    return 0;
}

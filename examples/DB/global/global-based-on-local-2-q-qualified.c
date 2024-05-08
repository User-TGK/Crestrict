int x;
int* p;

int* foo(int* restrict q) {
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

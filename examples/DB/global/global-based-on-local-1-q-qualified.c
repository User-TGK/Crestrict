int x;
int* p;

int foo(int* restrict q) {
    p = q;

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

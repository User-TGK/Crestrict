int x;
int* p;

int foo(int* restrict q) {
    *q = 10;
    *p = 11;
    return *q;
}

int main() {
    int* restrict _;
    int z;

    p = &x;
    z = foo(p);
    return 0;
}

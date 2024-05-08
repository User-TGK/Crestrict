int foo(int* q, int* r) {
    *q = 10;
    *r = 11;
    return *q;
}

int main() {
    int v[2];   
    int* restrict a[2];
    int x;

    a[0] = &v[0];
    a[1] = &v[1];

    foo(a[0], a[1] - 1);

    return 0;
}

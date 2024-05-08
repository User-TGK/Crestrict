// 3.3.5 (FN) Returning a restrict pointer

int* as_mut_ptr(int* restrict v) {
    return v;
}

int main() {
    int* restrict _;
    int a[5];

    int* p1;
    int* p2;

    p1 = as_mut_ptr(a);
    p2 = as_mut_ptr(a);

    *p1 = 0;
    *p2 = 0;

    return 0;
}

// 3.3.4 (FP) Indistinguishable restrict pointers

int main() {
    int x;
    int* restrict a[2];
    a[0] = &x;
    a[1] = &x;

    *(a[0]) = 10;
    *(a[1]) = 11;

    return 0;
}
int main () {
    int x;

    int* restrict p = &x;
    int* restrict q = &x;

    *p = 10;
    *q = 11;

    return 0;
}
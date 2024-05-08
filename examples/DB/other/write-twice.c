void f (int * restrict p) {
    *p = 0;
    *p = 1;
}

int main () {
    int x = 0;
    f(&x);

    return 0;
}

void bar(int* p) {
    int* q = p;
    int* r = p;

    *p = 5;
    *r = 6;
    *q = 4;
}

int main() {
    int x = 0;

    int* restrict p = &x;

    bar(p);

    return 0;
}

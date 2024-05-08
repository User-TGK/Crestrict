int foo(int *restrict * p, int *restrict * q)
{
    int* r = *p;

    *r = 10;
    **q = 11;
    return *r;
}

int main2() {
    int x = 0;
    int *restrict xp = &x;

    return foo(&xp, &xp);
}

int main() {
    return main2();
}

int foo(int *restrict * p, int *restrict * q)
{
    **p = 10;
    **q = 11;
    return **p;
}

int main3() {
    int x = 0;
    int* xp1 = &x;
    int* xp2 = &x;

    return foo(&xp1, &xp2);
}

int main() {
    return main3();
}

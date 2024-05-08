int foo(int *restrict *restrict p, int *restrict *restrict q)
{
    int* r = *p;

    *r = 10;
    **q = 11;
    return *r;
}

int main4() {
    int x = 0;
    int* xp = &x;

    return foo(&xp, &xp);
}

int main() {
    return main4();
}

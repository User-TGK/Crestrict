// 3.3.2 (FP) Nested restrict pointers

int foo(int *restrict *restrict p, int *restrict *restrict q) 
{
    **p = 10;
    **q = 11;

    return **p;
}

int main() {
    int x = 0;
    int* xp = &x;

    int res = foo(&xp, &xp);

    return 0;
}

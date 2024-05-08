int foo(int *restrict *restrict *restrict p, int *restrict *restrict *restrict q)
{
    int* r = **p;

    *r = 10;
    ***q = 11;
    return *r;
}

int main() {
    int x = 0;
    int* restrict xp = &x;
    int * restrict*restrict xpp = &xp;

    int res;                
    res = foo(&xpp, &xpp);

    return 0;
}
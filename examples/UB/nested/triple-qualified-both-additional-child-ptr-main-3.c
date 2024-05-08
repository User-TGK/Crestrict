int foo(int *restrict *restrict *restrict p, int *restrict *restrict *restrict q)
{
    int* r = **p;

    *r = 10;
    ***q = 11;
    return *r;
}

int main() {
    int x = 0;
    int* xp1 = &x;
    int* xp2 = &x;
    int** xpp1 = &xp1;
    int** xpp2 = &xp2;

    int res;                
    res = foo(&xpp1, &xpp2);

    return 0;
}
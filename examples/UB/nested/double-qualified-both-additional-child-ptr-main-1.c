int foo(int *restrict *restrict p, int *restrict *restrict q)
{
    int* r = *p;

    *r = 10;
    **q = 11;
    return *r;
}

int main1() {
    int x = 0;
    int *restrict xp1 = &x;
    int *restrict xp2 = &x;

    return foo(&xp1, &xp2);
}

int main() {
    return main1();
}

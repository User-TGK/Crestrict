int foo(int *restrict *restrict p, int** q)
{
    **p = 10;
    **q = 11;
    return **p;
}

int main1() {
    int x = 0;
    int *restrict xp1 = &x;
    int * xp2 = &x;

    return foo(&xp1, &xp2);
}

int main() {
    return main1();
}

int foo(int *restrict *restrict p, int** q)
{
    **p = 10;
    **q = 11;
    return **p;
}

int main4() {
    int x = 0;
    int* xp = &x;

    return foo(&xp, &xp);
}

int main() {
    return main4();
}

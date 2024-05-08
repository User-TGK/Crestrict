int bar(int* q)
{
    int* restrict r;
    // If we would not retain the base (lp, sifoo) in the reassignment to r 
    // at line 6, line 8 would be UB.
    r = q;

    return *r;
}

int foo(int* restrict p)
{
    *p = 10;

    return bar(p);
}

int main()
{
    int x;
    return foo(&x);
}

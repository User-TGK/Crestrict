int foo(int *restrict *restrict *restrict p, int *restrict *restrict *restrict q)
{
    ***p = 10;
    ***q = 11;
    return ***p;
}

int main() {
    int x = 0;
    int* xp = &x;
    int** xpp = &xp;

    int res;                
    res = foo(&xpp, &xpp);

    return 0;
}


// Evaluating ***p means
// (1) Loading from p, results in slxpp
// (2) Loading from slxpp, results in slxp
// (3) Loading from slxp, results in x
// If writing, i.e. at line 3/4, we thus write to x.

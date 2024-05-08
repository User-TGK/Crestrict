// Note: the block B refers to the declaration in which the restrict qualifier occurs.
// i.e., sifoo below.
//
// The restrict state in main will be restricted(∅) by filter bases.
//
void foo(int *restrict * xpp, int x) {
    **xpp = x;
}

int main() {
    int x;
    int *xp = &x;

    foo(&xp, 0);
    foo(&xp, 1);

    return 0;
}

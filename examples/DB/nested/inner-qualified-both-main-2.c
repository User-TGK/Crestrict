// No UB according to the GCC mailing list https://gcc.gnu.org/bugzilla/show_bug.cgi?id=14192#c8

// *p == *q is fine.
// So, we simply have two declarators both allowing to designate the same restrict
// object P = &*p = &*q.

int foo(int *restrict * p, int *restrict * q)
{
    **p = 10;
    **q = 11;
    return **p;
}

int main2() {
    int x = 0;
    int *restrict xp = &x;

    return foo(&xp, &xp);
}

int main() {
    return main2();
}

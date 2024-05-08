// RestrictCheck: DB
// KCC: DB
// Standard: DB

void f(int n, int* restrict p, int* restrict q)
{
    while (n > 0) {
        *p = *q;
        // Pull out the post decrements and increments.
        p = p + 1;
        q = q + 1;
        n = n - 1;
    }
}

void g()
{
    int d[100];
    for (int i = 0; i < 100; ++i)
    {
        d[i] = 0;
    }

    f(50, d + 50, d);
}

int main() {
    g();

    return 0;
}

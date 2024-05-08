void share_until_write(int* restrict u)
{
    int* x = u;
    int* y = u;
    int* z = u;

    int _v1 = *y;
    int _v2 = *x;
    int _v3 = *z;

    int* restrict w = u;
    *w = 42;

    int _v4 = *x;
}

int main() {
    int x = 5;                      // l:= &x, mem(l) = 5
    share_until_write(&x);

    return 0;
}
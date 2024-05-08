// https://github.com/kframework/c-semantics/blob/master/tests/unit-pass/j069.c

int *restrict p;
int *restrict q;

int f(int *restrict p, int *restrict q) {
    return 0;
}

int main(void) {
    int x = 5;
    int y = 6;
    p = &x;
    q = &y;

    {
        int *restrict a = p;
        int *restrict b = q;
    }

    return f(p, q);
}

// https://github.com/kframework/c-semantics/blob/master/tests/unit-fail/j069b.c

int x = 5;
int y = 5;
int * restrict p = &x;
int * restrict q = &y;

int main(void) {
    p = q;
    return 0;
}

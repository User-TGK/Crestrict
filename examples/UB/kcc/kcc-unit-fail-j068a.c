// https://github.com/kframework/c-semantics/blob/master/tests/unit-fail/j068a.c

int f(int *restrict a, int *restrict b) {
      *a = 1;
      *b = 1;
      return 0;
}

int main() {
      int a = 5;
      return f(&a, &a);
}

// https://github.com/kframework/c-semantics/blob/master/tests/unit-fail/j068b.c

int g(/*const*/ int *restrict a, int *b) {
      int x;
      *b = 1;
      x = *a; // expressions as statements not supported, so added a var decl
      return 0;
}

int main() {
      int a = 5;
      return g(&a, &a);
}

// https://github.com/kframework/c-semantics/blob/master/tests/unit-fail/j068c.c
// We moved it to DB instead of UB because we don't support type casts.

// this example is loosely based on an example in sentence 1509 in http://www.knosof.co.uk/cbook/cbook.html
int g(int * p) {
      *p = 10;
      return 0;
}

int f(/**const*/ int * restrict p) {
    // We don't support type casts (including dropping type qualifiers as attempted below)
    //   g((int*)p);
    g(p);  
    return *p;
}

int main(){
      int x = 5;
      return f(&x) == 5;
}

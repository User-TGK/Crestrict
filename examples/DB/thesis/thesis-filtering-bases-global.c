// Alternative to example 3.3.5 (returning a restrict pointer),
// global l-value based on local restrict pointer.

int x;
int* p;

void foo(int* restrict q) {
    p = q;
}

int main() {
    int* restrict _;

    p = &x;

    *p = 10;
    foo(&x);
    *p = 11;
    
    return 0;
}

struct bar {
    int* restrict p;
    int* restrict q;
};

int h(struct bar b) {
    *b.p = 10;
    *b.q = 11; 
    return *b.p; // Optimized by GCC to return 10. Clang has no support for restrict other than parameter declarations.
}

int main() {
    int x = 0;

    // This invocation makes b.p and b.q alias within the invocation of h,
    // and both access and modify x (thus UB).
    return h((struct bar){&x, &x});
}

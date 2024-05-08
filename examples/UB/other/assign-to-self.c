// Example by Ralf Jung: https://github.com/llvm/llvm-project/issues/73516

void assign(int* restrict p, int* restrict q) {
    *p = *q;
}

int main() {
    int x = 10;
    assign(&x, &x); // UB according to the standard: section 3.1 NOTE 2: ‘‘Modify’’ includes the case where the new value being stored is the same as the previous value.

    return 0;
}

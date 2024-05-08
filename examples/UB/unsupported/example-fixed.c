void f(int* restrict u) {
    int* restrict x = u;
    {
        int* restrict y = x;
        
        *y = 12;
        *x = 10;
    }
    
}

int main() {
    int a = 0;

    f(&a);

    return 0;
}
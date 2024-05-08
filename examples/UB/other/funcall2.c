int y = 0;

void g () {
    y++;
}

int f(int* restrict x) {
    int a = *x;
    g();

    return *x + a;
}

int main() {
    int b;
    b = f(&y);
    return b;
}

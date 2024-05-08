int y = 0;
int z = -1;

void g () {
    z = y;
}

void f(int* restrict x) {
    g();

    *x = *x + 1;
}

int main() {
    f(&y);

    return z;
}

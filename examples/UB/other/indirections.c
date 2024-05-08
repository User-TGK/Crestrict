// int foo(int* restrict p) {
//     int* q = p;

//     *q = 3;
//     int y = *p;
//     return *p;
// }

int main() {
    int x = 0;

    int* restrict y = &x;
    int* q = y;
    int* r = q;
    int a;
    r = &x; // Based on is removed.

    *q = 3;
    *r = 4;
    a = *y;

    return 0;
}

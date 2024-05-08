

struct Vec {
    int* restrict p;
};

int* as_mut_ptr(struct Vec v) {
    return v.p;
}

int main() {
    int* restrict _;
    int array[5];

    struct Vec v = {array};

    int* p1 = as_mut_ptr(v);
    int* p2 = as_mut_ptr(v);

    *p1 = 0;
    *p2 = 0;

    return 0;
}
// Similar to based-on-nested, but the restrict qualifier 
// is not part of the nested declaration but a separate declaration.

int main() {
    int x;  // Object X
    int* r; // Uninitialized int ptr
    int** p = &r; // Pointer to pointer to int (pointer to `r`)

    // Restrict int pointer to x 
    int* restrict q = &x;

    // r now points to x, *and* is based on q.
    r = q;

    // Write to `x` via p and r. Because `r` is based on q, this is fine.
    **p = 5;

    // Read via `q`.
    return *q;
}

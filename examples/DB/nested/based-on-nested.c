int main() {
    int x;                      // Object X
    int* xp = &x;               // Restrict qualified object XP
    int*restrict * xpp = &xp;   // Pointer to XP

    int* q = *xpp;              // q is a pointer to X with base XP

    **xpp = 5;                  // Load to X with base XP 
    *q = 4;                     // Load to X with base XP

    return 0;
}

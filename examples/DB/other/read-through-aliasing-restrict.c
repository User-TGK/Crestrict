// The problem of aliasing reads under the KCC semantics.
// The call to `g` means that within scope f the state of sly becomes Unrestricted.
// This cannot be joined with the state of sly within the scope main, as this is Restricted.
// The restrict pointer declared is main is needed to force the semantic checks,
// whereas it clearly doesn't affect accesses to y whatsoever.

int y;                  // Stored at sly.
int z = 0;

void g () {
    z = y;
}

int f(int* restrict x) {
    g();                    // M(x) = ptr(sly, (lx, sif)) 

    return *x;              // Read: R = [ (sif, {sly -> onlyread({(slx, sif)})}), (simain, {sly -> restricted() }) ]
}

int main() {
    int* restrict _;    // again the restrict ptr tweak to prevent some invalid performance optimization
    y = 0;              // separate assignment instead of initialization
                        // sly = restricted(/emtpyset).
    f(&y);

    return z;
}

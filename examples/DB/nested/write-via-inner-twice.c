// https://gcc.gnu.org/pipermail/gcc/2024-February/243322.html
// xpp1 = xpp2 -> *xpp1 = *xpp2

int main() {
    int x;                    
    int *xp = &x;             
    int *restrict *xpp1 = &xp;
    int *restrict *xpp2 = &xp;

    **xpp1 = 5;
    **xpp2 = 6;

    return 0;
}

int main() {
    int x = 1;                
    int *xp = &x;             
    int *restrict *xpp1 = &xp;
    int *restrict *xpp2 = &xp;

    return **xpp1 + **xpp2;
}

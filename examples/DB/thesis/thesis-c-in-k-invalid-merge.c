// 3.3.1 (FN Aliasing Loads)

void h(int* q, int* restrict r, int* restrict s)
{
    // Assume $r$ is stored at $b_r$ and $s$ at $b_s$.
    // $M(b_r) = \mathsf{ptr}(sl, [(b_p, 0), (b_r, 1)])$
    // $M(b_s) = \mathsf{ptr}(sl, [(b_p, 0), (b_s, 1)])$
    *q = *r + *s;
}

// $\mathsf{Scope}_0, R = [(0, \emptyset)]$ $\label{lst:example-state-merge-main}$
int main()
{
    int x;
    int y; // y is stored at location sl
    int* restrict p = &y; // Assume $p$ is stored at $b_p$. $M(b_p) = \mathsf{ptr}(sl, (b_p, 0)) $

    *p = 0; // $\mathsf{Scope}_0, R = [(0, [sl \mapsto \mathsf{Restricted}([(b_p, 0)])])]$

    // Defined behavior because b is not modified.
    h(&x, p, p);

    return 0;
}

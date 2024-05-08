// Scope $\scope{main}, R = [(\scope{main}, \emptyset)]$ $\label{lst:example-scopes-main}$
int main() {
    int x = 5;            // $x$ is stored at $(b_x, 0)$
    int* restrict p = &x; // $p$ is stored at $(b_p, 0)$
                          // $M(b_p, 0) = \ptr{(b_x, 0), \set{(b_p,\scope{main})}}$
    int y = *p; // Load via $p$, $R = [(\scope{main}, \set{(b_x, 0) \mapsto \onlyread{\set{(b_p, \scope{main})}}})]$

    // {
        int* q = &x;
        *q = 0; // Load via $q$

        while (1) {continue;}
    // }

    return y;
}
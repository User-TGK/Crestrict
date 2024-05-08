// 3.3.3 (FP) Semantic preservation under inlining

// Scope $\scope{foo}, R = [(\scope{foo}, \emptyset), (\scope{main}, \set{(b_x, 0) \mapsto \onlyread{\set{(b_p, \scope{main})}}})]$
void foo(int* q) {
    
    *q = 0; // Load via $q$,
            // $R = [(\scope{foo}, \set{(b_x, 0) \mapsto \restricted{\emptyset}}), (\scope{main}, \set{(b_x, 0) \mapsto \onlyread{\set{(b_p, \scope{main})}}})]$$\label{lst:example-scopes-conflict}$

    while (1) {continue;}
}

// Scope $\scope{main}, R = [(\scope{main}, \emptyset)]$ $\label{lst:example-scopes-main}$
int main() {
    int x = 5;            // $x$ is stored at $(b_x, 0)$
    int* restrict p = &x; // $p$ is stored at $(b_p, 0)$
                          // $M(b_p, 0) = \ptr{(b_x, 0), \set{(b_p,\scope{main})}}$
    int y = *p; // Load via $p$, $R = [(\scope{main}, \set{(b_x, 0) \mapsto \onlyread{\set{(b_p, \scope{main})}}})]$

    foo(&x); // As $(b_p, 0)$ is non-local to $\mathtt{foo}$,
             // we attempt to merge the state depicted at line $\ref{lst:example-scopes-conflict}$
             // UB: $\mathtt{filter\_bases}(\restricted{\emptyset}) \joinsym \onlyread{\set{(b_p, \scope{main})}} = \rsub$
    return y;
}
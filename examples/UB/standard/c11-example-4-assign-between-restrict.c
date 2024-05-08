/**
 * EXAMPLE 4 (Source: C99 standard)
 *
 * ``` The rule limiting assignments between restricted pointers does not distinguish between a
 * function call and an equivalent nested block. With one exception, only ‘‘outer-to-inner’’
 * assignments between restricted pointers declared in nested blocks have defined behavior. ```
 */

int main()
{
    int *restrict p1;
    int *restrict q1;
    p1 = q1; // undefined behavior
    {
        int *restrict p2 = p1; // valid
        int *restrict q2 = q1; // valid
        p1 = q2; // undefined behavior
        p2 = q2; // undefined behavior
    }
    return 0;
}

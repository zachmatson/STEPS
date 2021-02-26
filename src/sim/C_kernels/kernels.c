#include <stddef.h>
#include <math.h>

void grow_lineages_inplace_c(size_t len, double N[len], double W[len], double delta_t) {
    for (size_t i = 0; i < len; ++i) {
        N[i] *= exp2(W[i] * delta_t);
        N[i] = ceil(N[i]);
    }
}

#include <stddef.h>
#include <math.h>

void grow_lineages_inplace_c(size_t len, double N[len], double W[len], double delta_t) {
    #ifdef KERNELS_USE_AVX2
        const double log2 = log(2);
        #pragma omp simd
        for (size_t i = 0; i < len; ++i) {
            N[i] *= exp(log2 * W[i] * delta_t);
            N[i] = ceil(N[i]);
        }
    #else
        #pragma omp simd
        for (size_t i = 0; i < len; ++i) {
            N[i] *= exp2(W[i] * delta_t);
            N[i] = ceil(N[i]);
        } 
    #endif
}

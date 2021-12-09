#include <stddef.h>
#ifdef KERNELS_USE_AVX2
    #include <immintrin.h>
    #include "sleef.h"
#endif

// See build.rs for how this gets used
// Basically, we need this to be in C because Rust does not have a stable way
// to use FFI with SIMD types, which we need to use the SLEEF exp2 function

// Rust will add its own libm when linking
// We might not be able to get math headers on some targets (e.g. WASM)
double exp2(double);

/// Grow the lineage sizes in N according to the fitnesses in W and timestep delta_t
/// Grows according to the formula new_N = old_N * 2^(W * delta_t)
void grow_lineages_inplace_c(size_t len, double *N, const double *W, const double delta_t) {
    #ifdef KERNELS_USE_AVX2
        const size_t VEC_LEN = 4;
        const size_t main_len = len / VEC_LEN;
        const size_t remainder_len = len % VEC_LEN;
        const __m256d delta_t_packed = _mm256_set1_pd(delta_t);

        for (size_t i = 0; i < main_len; ++i) {
            __m256d N_vec = _mm256_loadu_pd(N);
            const __m256d W_vec = _mm256_loadu_pd(W);

            __m256d temp = _mm256_mul_pd(W_vec, delta_t_packed);
           temp = Sleef_exp2d4_u10avx2(temp);
           temp = _mm256_mul_pd(N_vec, temp);
           _mm256_storeu_pd(N, temp);

           N += VEC_LEN;
           W += VEC_LEN;
        }

        len = remainder_len;
    #endif
    for (size_t i = 0; i < len; ++i) {
        N[i] *= exp2(W[i] * delta_t);
    }
}

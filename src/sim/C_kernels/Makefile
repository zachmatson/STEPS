all: kernels

kernels: kernels.c
	clang -O3 -fveclib=SVML -march=skylake -c kernels.c -o ${OUT_DIR}/kernels.o
	ar -r ${OUT_DIR}/librellteekernels.a ${OUT_DIR}/kernels.o
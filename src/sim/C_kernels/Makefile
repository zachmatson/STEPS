ifeq ($(KERNELS_USE_AVX2),1)
  $(info Compiling kernels with AVX2)
  EXTRA_FLAGS = -mavx2 -mtune=skylake -D KERNELS_USE_AVX2
else
  EXTRA_FLAGS = 
endif

kernels: kernels.c
	gcc -lm -fopenmp -O3 -ffast-math $(EXTRA_FLAGS) -o ${OUT_DIR}/kernels.o -c kernels.c
	ar -rcs ${OUT_DIR}/librellteekernels.a ${OUT_DIR}/kernels.o
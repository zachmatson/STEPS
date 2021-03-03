ifeq ($(KERNELS_USE_AVX2),1)
  $(info Compiling kernels with AVX2)
  EXTRA_FLAGS = -mavx2 -mtune=skylake -D KERNELS_USE_AVX2 -I ${SLEEF_HEADER_DIR}
else
  EXTRA_FLAGS = 
endif

kernels: kernels.c
	gcc -c -fPIC -O3 $(EXTRA_FLAGS) -Wall -Wextra -Wpedantic -o ${OUT_DIR}/kernels.o kernels.c
	ar -rcs ${OUT_DIR}/librellteekernels.a ${OUT_DIR}/kernels.o

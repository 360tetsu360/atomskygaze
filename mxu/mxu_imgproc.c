#include <stdio.h>
#include <stdint.h>
#include "mxu2.h"
#include "mxu_imgproc.h"

#define VECTOR_SIZE 16

void buffer_diff(int8_t* input1, int8_t* input2, int8_t* out, size_t length) {
    size_t i;
    for (i = 0; i < length; i += VECTOR_SIZE) {
        v16i8 vec1 = _mx128_lu1q(&input1[i], 0);
        v16i8 vec2 = _mx128_lu1q(&input2[i], 0);
        v16i8 result = _mx128_sub_b(vec1, vec2);
        _mx128_su1q(result, &out[i], 0);
    }
}


// Much faster than OpenCV absdiff()!!!!!
void buffer_absdiff(uint8_t* input1, uint8_t* input2, uint8_t* out, size_t length) {
    size_t i;
    for (i = 0; i < length; i += VECTOR_SIZE) {
        v16u8 vec1 = (v16u8)_mx128_lu1q(&input1[i], 0);
        v16u8 vec2 = (v16u8)_mx128_lu1q(&input2[i], 0);
        v16u8 max = _mx128_maxu_b(vec1, vec2);
        v16u8 min = _mx128_minu_b(vec1, vec2);
        v16i8 result = (v16i8)_mx128_subuu_b(max, min);
        _mx128_su1q(result, &out[i], 0);
    }
}

void buffer_add(uint8_t* input1, uint8_t* input2, uint8_t* out, size_t length) {
    size_t i;
    for (i = 0; i < length; i += VECTOR_SIZE) {
        v16u8 vec1 = (v16u8)_mx128_lu1q(&input1[i], 0);
        v16u8 vec2 = (v16u8)_mx128_lu1q(&input2[i], 0);
        v16i8 result = (v16i8)_mx128_adduu_b(vec1, vec2);
        _mx128_su1q(result, &out[i], 0);
    }
}
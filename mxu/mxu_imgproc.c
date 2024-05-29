#include <stdio.h>
#include <stdint.h>
#include <string.h> /* memset */
#include <unistd.h> /* close */
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

void buffer_div_add(uint8_t** input_list, uint8_t* out, uint8_t v, size_t length) {
    size_t i, j;
    uint8_t div_v[VECTOR_SIZE];

    memset(div_v, v, VECTOR_SIZE);

    v16u8 div_vec = (v16u8)_mx128_lu1q(div_v, 0);
    for (i = 0; i < length; i += VECTOR_SIZE) {
        v16u8 dst = (v16u8)_mx128_li_b(0);
        v16u8 mod_dst = (v16u8)_mx128_li_b(0);
        for (j = 0; j < (size_t)v; j++) {
            v16u8 vec1 = (v16u8)_mx128_lu1q(&input_list[j][i], 0);
            v16u8 comp = _mx128_divu_b(vec1, div_vec);
            v16u8 mod = _mx128_modu_b(vec1, div_vec);
            dst = _mx128_adduu_b(comp, dst);
            mod_dst = _mx128_adduu_b(mod, mod_dst);
        }
        v16u8 mod_comp = _mx128_divu_b(mod_dst, div_vec);
        dst = _mx128_adduu_b(mod_comp, dst);
        _mx128_su1q((v16i8)dst, &out[i], 0);
    }
}

void buffer_max(uint8_t** input_list, uint8_t* out, size_t frame_len, size_t length) {
    size_t i, j;

    for (i = 0; i < length; i += VECTOR_SIZE) {
        v16u8 dst = (v16u8)_mx128_li_b(0);
        for (j = 0; j < frame_len; j++) {
            v16u8 vec1 = (v16u8)_mx128_lu1q(&input_list[j][i], 0);
            dst = _mx128_maxu_b(dst, vec1);
        }
        _mx128_su1q((v16i8)dst, &out[i], 0);
    }
}

void fast_memcpy(uint8_t* src, uint8_t* dst, size_t length) {
    size_t i;
    for (i = 0; i < length; i += VECTOR_SIZE) {
        v16i8 vec = _mx128_lu1q(&src[i], 0);
        _mx128_su1q(vec, &dst[i], 0);
    }
}
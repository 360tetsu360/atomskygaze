#include <stdio.h>
#include <stdint.h>
#include <string.h> /* memset */
#include <unistd.h> /* close */
#include <math.h>
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
        v16i8 result = _mx128_subua_b(vec1, vec2);
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

void buffer_max_list(uint8_t** input_list, uint8_t* out, size_t frame_len, size_t length) {
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

void lighten_stack(uint8_t* src, uint8_t* dst, size_t length) {
    size_t i;
    for (i = 0; i < length; i += VECTOR_SIZE) {
        v16u8 vec1 = (v16u8)_mx128_lu1q(&src[i], 0);
        v16u8 vec2 = (v16u8)_mx128_lu1q(&dst[i], 0);
        v16u8 vec3 = _mx128_maxu_b(vec1, vec2);
        _mx128_su1q((v16i8)vec3, &dst[i], 0);
    }
}

void fast_memcpy(uint8_t* src, uint8_t* dst, size_t length) {
    size_t i;
    for (i = 0; i < length; i += VECTOR_SIZE) {
        v16i8 vec = _mx128_lu1q(&src[i], 0);
        _mx128_su1q(vec, &dst[i], 0);
    }
}

void fast_mean_stddev(uint8_t* src, size_t length, double* mean, double* stddev) {
    size_t i;
    uint64_t sum_buf[2], ssd_buf[2];
    uint8_t mean_buff[VECTOR_SIZE];

    v16u8 unit_vec_u8 = (v16u8)_mx128_li_b(1);
    v8u16 unit_vec_u16 = (v8u16)_mx128_li_h(1);
    v4u32 unit_vec_u32 = (v4u32)_mx128_li_w(1);
    v2u64 sum = (v2u64)_mx128_li_d(0);
    v2u64 ssd = (v2u64)_mx128_li_d(0);

    for (i = 0; i < length; i += VECTOR_SIZE) {
        v16u8 vec = (v16u8)_mx128_lu1q(&src[i], 0);
        v8u16 sum_u16 = _mx128_dotpu_h(vec, unit_vec_u8);
        v4u32 sum_u32 = _mx128_dotpu_w(sum_u16, unit_vec_u16);
        sum = _mx128_daddu_d(sum, sum_u32, unit_vec_u32);
    }

    _mx128_su1q((v16i8)sum, (uint8_t*)sum_buf, 0);
    *mean = (sum_buf[0] + sum_buf[1]) / length;
    memset(mean_buff, (uint8_t)*mean, VECTOR_SIZE);
    v16u8 mean_vec = (v16u8)_mx128_lu1q(mean_buff, 0);

    for (i = 0; i < length; i += VECTOR_SIZE) {
        v16u8 vec = (v16u8)_mx128_lu1q(&src[i], 0);
        v16u8 diff = (v16u8)_mx128_subua_b(vec, mean_vec);
        v8u16 sum_u16 = _mx128_dotpu_h(diff, diff);
        v4u32 sum_u32 = _mx128_dotpu_w(sum_u16, unit_vec_u16);
        ssd = _mx128_daddu_d(ssd, sum_u32, unit_vec_u32);
    }

    _mx128_su1q((v16i8)ssd, (uint8_t*)ssd_buf, 0);
    *stddev = sqrt((ssd_buf[0] + ssd_buf[1]) / length);
}

void create_mask(const uint8_t* mask_small, uint8_t* mask) {
    size_t x, y;
    uint8_t buff[VECTOR_SIZE];
    for( y = 0; y < 18; y++) {
        for( x = 0; x < 640; x += 16) {
            size_t mod = x % 20;
            size_t first_index = (x / 20) + y*32;
            size_t change_index = 20 - mod;
            v16u8 zero = (v16u8)_mx128_mfcpu_b(0);
            _mx128_su1q((v16i8)zero, buff, 0);
            if(mask_small[first_index]) {
                int i;
                for(i=0; i < change_index; i++) {
                    buff[i] = 1;
                }
            }
            if(change_index < 16) {
                if(mask_small[first_index+1]) {
                    int i;
                    for(i=change_index; i < 16; i++) {
                        buff[i] = 1;
                    }
                }
            }

            int j;
            v16i8 vec = _mx128_lu1q(buff, 0);
            for(j = 0; j < 20; j++) {
                _mx128_su1q(vec, &mask[x+640*(y*20+j)], 0);
            }
        }
    }
}
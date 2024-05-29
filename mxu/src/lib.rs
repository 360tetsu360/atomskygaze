#[link(name = "mxu_imgproc")]
extern "C" {
    /// input1 - input2
    pub fn buffer_diff(input1: *const i8, input2: *const i8, out: *mut i8, size: usize);

    /// abs(input1 - input2)
    pub fn buffer_absdiff(input1: *const u8, input2: *const u8, out: *mut u8, size: usize);

    /// input1 + input2
    pub fn buffer_add(input1: *const u8, input2: *const u8, out: *mut u8, size: usize);

    /// Sum buffer
    pub fn buffer_add_list(
        input: *const *const u8,
        out: *mut u8,
        buf_size: usize,
        list_size: usize,
    );

    /// output = sum(input_list) / len(input_list)
    pub fn buffer_div_add(input: *const *const u8, out: *mut u8, v: u8, size: usize);

    /// output = max(input_list)
    pub fn buffer_max_list(input: *const *const u8, out: *mut u8, frame_len: usize, size: usize);

    /// output = max(input, output)
    pub fn lighten_stack(src: *const u8, out: *mut u8, size: usize);

    /// src -> dst
    pub fn fast_memcpy(src: *const u8, out: *mut u8, size: usize);
}

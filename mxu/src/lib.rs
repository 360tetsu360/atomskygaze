#[link(name = "mxu_imgproc")]
extern "C" {
    /// input1 - input2
    pub fn buffer_diff(input1: *const i8, input2: *const i8, out: *mut i8, size: usize);

    /// abs(input1 - input2)
    pub fn buffer_absdiff(input1: *const u8, input2: *const u8, out: *mut u8, size: usize);

    /// input1 + input2
    pub fn buffer_add(input1: *const u8, input2: *const u8, out: *mut u8, size: usize);
    
    /// output += input / v
    pub fn buffer_div_add(input: *const *const u8, out: *mut u8, v: u8, size: usize);

    /// src -> dst
    pub fn fast_memcpy(src: *const u8, out: *mut u8, size: usize);
}

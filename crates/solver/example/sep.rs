
fn main() {
    let mut fptr = fitsio::FitsFile::open("/media/mmc/wide.fits").unwrap();
    let hdu = fptr.primary_hdu().unwrap();

    let shape = match hdu.info {
        HduInfo::ImageInfo { ref shape, .. } => shape,
        _ => panic!("Unexpected hdu type"),
    };

    println!("w: {}, h: {}", shape[0], shape[1]);

    let mut image_data: Vec<u32> = hdu.read_image(&mut fptr).unwrap();

}

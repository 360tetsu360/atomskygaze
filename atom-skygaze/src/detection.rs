use fitsio::images::{ImageDescription, ImageType};
use fitsio::FitsFile;
use isvp_sys::*;
use log::{error, info};
use opencv::core::*;
use opencv::imgcodecs::imwrite;
use opencv::imgproc;
use opencv::Error;
use std::os::raw::c_void;
use std::time::{SystemTime, UNIX_EPOCH};
use opencv::prelude::BackgroundSubtractorTrait;

fn brightest_frame(imgs: &[Mat]) -> Result<Mat, Error> {
    let mut ret = imgs[0].clone();

    for img in &imgs[1..] {
        let mut tmp = Mat::default();
        max(&ret, img, &mut tmp)?;
        ret = tmp;
    }

    Ok(ret)
}

fn make_diff_list(imgs: &[Mat]) -> Result<Vec<Mat>, Error> {
    let mut diff_list = vec![];

    for win in imgs.windows(2) {
        let img1 = &win[1];
        let img2 = &win[0];

        let mut diff = Mat::default();
        subtract(img1, img2, &mut diff, &no_array(), -1)?;

        diff_list.push(diff);
    }

    Ok(diff_list)
}

fn detect_lines(img: &Mat, min_length: f64) -> Result<Vector<Vec4i>, Error> {
    let mut blur = Mat::default();
    imgproc::gaussian_blur(img, &mut blur, Size::new(3, 3), 0., 0., BORDER_DEFAULT)?;

    let mut canny = Mat::default();
    imgproc::canny(&blur, &mut canny, 33., 66., 3, false)?;

    let mut lines: Vector<Vec4i> = Vector::new();
    imgproc::hough_lines_p(
        &canny,
        &mut lines,
        1.,
        std::f64::consts::PI / 180.,
        10,
        min_length,
        3.,
    )?;

    Ok(lines)
}

pub unsafe fn detection_init() -> bool {
    if IMP_FrameSource_SetFrameDepth(1, 10) < 0 {
        error!("IMP_FrameSource_SetFrameDepth failed");
        return false;
    }

    return true;
}

pub unsafe fn detection_start() {
    let mut last_frame_time = 0;
    let mut old_frame: *mut IMPFrameInfo = std::ptr::null_mut();
    let mut old_gray = Mat::default();
    let mut i = 0;
    loop {
        let mut new_frame: *mut IMPFrameInfo = std::ptr::null_mut();
        IMP_FrameSource_GetFrame(1, &mut new_frame);
        println!("{}, duration: {}", (*new_frame).timeStamp, (*new_frame).timeStamp - last_frame_time);
        last_frame_time = (*new_frame).timeStamp;

        let start = std::time::Instant::now();

        let (img_width, img_height) = ((*new_frame).width, (*new_frame).height);
        let new_gray = Mat::new_rows_cols_with_data_def(
            img_height as i32,
            img_width as i32,
            CV_8UC1,
            (*new_frame).virAddr as *mut c_void,
        )
        .unwrap();

        if old_frame != std::ptr::null_mut() {
            let mut diff = Mat::default();

            absdiff(&new_gray, &old_gray, &mut diff).unwrap();

            let (mean, stddev) : (f64, f64) = {
                let mut mean = Scalar_::default();
                let mut stddev = Scalar_::default();
                mean_std_dev(&diff, &mut mean, &mut stddev, &no_array()).unwrap();
                (mean[0], stddev[0])
            };

            let thresh_val = mean + stddev * 2.;

            if thresh_val > 1. {

            let mut blur = Mat::default();
            imgproc::gaussian_blur(&diff, &mut blur, Size::new(5, 5), 0., 0., BORDER_DEFAULT).unwrap();

            println!("mean {}, stddev {}, threshold {}", mean, stddev, thresh_val);
            let mut thresh = Mat::default();
            imgproc::threshold(
                &blur,
                &mut thresh,
                thresh_val,
                255.,
                imgproc::THRESH_BINARY,
            ).unwrap();

            if i > 30 {

            let data: Vec<u8> = thresh
                        .to_vec_2d::<u8>()
                        .unwrap()
                        .iter()
                        .flat_map(|v| v.iter())
                        .cloned()
                        .collect();
                    dbg!(data.len());

                    let primary_hdu_description = ImageDescription {
                        data_type: ImageType::UnsignedByte,
                        dimensions: &[360, 640],
                    };

                    let mut fitsfile = FitsFile::create(&format!("/media/mmc/detected_{}.fits", i))
                        .with_custom_primary(&primary_hdu_description)
                        .overwrite()
                        .open()
                        .unwrap();

                    let hdu = fitsfile.primary_hdu().unwrap();

                    hdu.write_image(&mut fitsfile, &data).unwrap();
                    fitsfile.pretty_print().unwrap();

                    println!("write to detected_{}.jpg", i);
                }

                    if i == 60 {
                        break;
                    }
                    i += 1;


            }
        }

        IMP_FrameSource_ReleaseFrame(1, old_frame);
        old_frame = new_frame;
        old_gray = new_gray;

        println!("elapsed {}", std::time::Instant::now().duration_since(start).as_micros());
    }
}

/*
pub unsafe fn detection_start() {
    let mut old_frame: *mut IMPFrameInfo = std::ptr::null_mut();
    let mut old_gray = Mat::default();
    for i in 0..10 {
        let mut new_frame: *mut IMPFrameInfo = std::ptr::null_mut();
        IMP_FrameSource_GetFrame(1, &mut new_frame);

        let (img_width, img_height) = ((*new_frame).width, (*new_frame).height);
        let new_gray = Mat::new_rows_cols_with_data_def(
            img_height as i32,
            img_width as i32,
            CV_8UC1,
            (*new_frame).virAddr as *mut c_void,
        )
        .unwrap();

        if old_frame != std::ptr::null_mut() {
            let mut diff = Mat::default();
            subtract(&new_gray, &old_gray, &mut diff, &no_array(), -1).unwrap();

            let mut blur = Mat::default();
            imgproc::gaussian_blur(&diff, &mut blur, Size::new(3, 3), 0., 0., BORDER_DEFAULT).unwrap();

            /*for x in 0..4 {
                for y in 0..4 {
                    let cropped = Mat::roi(
                        &diff,
                        opencv::core::Rect {
                            x: x * 160,
                            y: y * 90,
                            width: 160,
                            height: 90,
                        },
                    )
                    .unwrap();*/

                    let (mean, stddev) : (f64, f64) = {
                        let mut mean = Scalar_::default();
                        let mut stddev = Scalar_::default();
                        mean_std_dev(&diff, &mut mean, &mut stddev, &no_array()).unwrap();
                        (mean[0], stddev[0])
                    };

                    let thresh_val = mean + stddev * 2.;
                    println!("mean {}, stddev {}, threshold {}", mean, stddev, thresh_val);
                    let mut thresh = Mat::default();
                    imgproc::threshold(
                        &diff,
                        &mut thresh,
                        thresh_val,
                        255.,
                        imgproc::THRESH_BINARY,
                    ).unwrap();

                    let data: Vec<u8> = thresh
                        .to_vec_2d::<u8>()
                        .unwrap()
                        .iter()
                        .flat_map(|v| v.iter())
                        .cloned()
                        .collect();
                    dbg!(data.len());

                    let primary_hdu_description = ImageDescription {
                        data_type: ImageType::UnsignedByte,
                        dimensions: &[360, 640],
                    };

                    let mut fitsfile = FitsFile::create(&format!("/media/mmc/detected_{}.jpg.fits", i))
                        .with_custom_primary(&primary_hdu_description)
                        .overwrite()
                        .open()
                        .unwrap();

                    let hdu = fitsfile.primary_hdu().unwrap();

                    hdu.write_image(&mut fitsfile, &data).unwrap();
                    fitsfile.pretty_print().unwrap();

                    println!("write to detected_{}.jpg", i);
                /*
                }
            }*/
            IMP_FrameSource_ReleaseFrame(1, old_frame);
        }

        old_frame = new_frame;
        old_gray = new_gray;
    }
}
*/
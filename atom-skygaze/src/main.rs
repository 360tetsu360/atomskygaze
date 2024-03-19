use fitsio::images::{ImageDescription, ImageType};
use fitsio::FitsFile;
use isvp_sys::*;
use opencv::core::*;
use opencv::imgproc;
use opencv::Error;
use std::os::raw::c_void;

const SENSOR_NAME: &[u8] = b"gc2053";

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

unsafe fn init_imp(sensor_info: *mut IMPSensorInfo) {
    dbg!(IMP_ISP_Open());
    dbg!(IMP_ISP_AddSensor(sensor_info));
    dbg!(IMP_ISP_EnableSensor());
    dbg!(IMP_System_Init());
    dbg!(IMP_ISP_EnableTuning());
    dbg!(IMP_ISP_Tuning_SetContrast(128));
    dbg!(IMP_ISP_Tuning_SetSharpness(128));
    dbg!(IMP_ISP_Tuning_SetSaturation(128));
    dbg!(IMP_ISP_Tuning_SetBrightness(128));
    dbg!(IMP_ISP_Tuning_SetISPRunningMode(
        IMPISPRunningMode_IMPISP_RUNNING_MODE_DAY
    ));
}

unsafe fn exit_imp(sensor_info: *mut IMPSensorInfo) {
    dbg!(IMP_System_Exit());
    dbg!(IMP_ISP_DisableSensor());
    dbg!(IMP_ISP_DelSensor(sensor_info));
    dbg!(IMP_ISP_DisableTuning());
    dbg!(IMP_ISP_Close());
}

unsafe fn imp() {
    println!("Hello imp");
    let mut sensor_info = IMPSensorInfo {
        name: [0i8; 32],
        cbus_type: IMPSensorControlBusType_TX_SENSOR_CONTROL_INTERFACE_I2C,
        __bindgen_anon_1: IMPSensorInfo__bindgen_ty_1 {
            i2c: IMPI2CInfo {
                type_: [0i8; 20],
                addr: 0x37,
                i2c_adapter_id: 0,
            },
        },
        rst_gpio: 0,
        pwdn_gpio: 0,
        power_gpio: 0,
    };
    sensor_info.name[..SENSOR_NAME.len()]
        .copy_from_slice(std::mem::transmute::<&[u8], &[i8]>(SENSOR_NAME));
    sensor_info.__bindgen_anon_1.i2c.type_[..SENSOR_NAME.len()]
        .copy_from_slice(std::mem::transmute::<&[u8], &[i8]>(SENSOR_NAME));

    init_imp(&mut sensor_info);

    let mut chn_0 = IMPFSChnAttr {
        picWidth: 1920,
        picHeight: 1080,
        pixFmt: IMPPixelFormat_PIX_FMT_NV12,
        crop: IMPFSChnCrop {
            enable: 1,
            left: 0,
            top: 0,
            width: 1920,
            height: 1080,
        },
        scaler: IMPFSChnScaler {
            enable: 0,
            outwidth: 0,
            outheight: 0,
        },
        outFrmRateNum: 25,
        outFrmRateDen: 1,
        nrVBs: 2,
        type_: IMPFSChnType_FS_PHY_CHANNEL,
        fcrop: IMPFSChnCrop {
            enable: 0,
            left: 0,
            top: 0,
            width: 0,
            height: 0,
        },
    };

    let mut chn_1 = IMPFSChnAttr {
        picWidth: 640,
        picHeight: 360,
        pixFmt: IMPPixelFormat_PIX_FMT_NV12,
        crop: IMPFSChnCrop {
            enable: 0,
            left: 0,
            top: 0,
            width: 1920,
            height: 1080,
        },
        scaler: IMPFSChnScaler {
            enable: 1,
            outwidth: 640,
            outheight: 360,
        },
        outFrmRateNum: 25,
        outFrmRateDen: 1,
        nrVBs: 2,
        type_: IMPFSChnType_FS_PHY_CHANNEL,
        fcrop: IMPFSChnCrop {
            enable: 0,
            left: 0,
            top: 0,
            width: 0,
            height: 0,
        },
    };

    dbg!(IMP_FrameSource_CreateChn(0, &mut chn_0));
    dbg!(IMP_FrameSource_SetChnAttr(0, &chn_0));
    dbg!(IMP_FrameSource_CreateChn(1, &mut chn_1));
    dbg!(IMP_FrameSource_SetChnAttr(1, &chn_1));

    dbg!(IMP_FrameSource_EnableChn(0));
    dbg!(IMP_FrameSource_EnableChn(1));
    dbg!(IMP_FrameSource_SetFrameDepth(0, 1));
    dbg!(IMP_FrameSource_SetFrameDepth(1, 10));

    let mut detections = 0;

    for _ in 0..10000 {
        let mut frames = vec![];
        let mut imgs = vec![];
        for _ in 0..5 {
            let mut frame_bak: *mut IMPFrameInfo = std::ptr::null_mut();
            dbg!(IMP_FrameSource_GetFrame(1, &mut frame_bak));

            let (img_width, img_height) = ((*frame_bak).width, (*frame_bak).height);
            //println!("width : {}, height {}", img_width, img_height);

            let gray = Mat::new_rows_cols_with_data_def(
                img_height as i32,
                img_width as i32,
                CV_8UC1,
                (*frame_bak).virAddr as *mut c_void,
            )
            .unwrap();

            frames.push(frame_bak);
            imgs.push(gray);
        }

        let diff_list = make_diff_list(&imgs).unwrap();

        let brightest = brightest_frame(&diff_list).unwrap();

        let detected = detect_lines(&brightest, 3.).unwrap();

        for line in detected {
            dbg!(line);
            detections += 1;
        }

        while let Some(frame_bak) = frames.pop() {
            dbg!(IMP_FrameSource_ReleaseFrame(1, frame_bak));
        }

        if detections > 100 {
            let data: Vec<u8> = brightest
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

            let mut fitsfile = FitsFile::create("/media/mmc/detected.fits")
                .with_custom_primary(&primary_hdu_description)
                .overwrite()
                .open()
                .unwrap();

            let hdu = fitsfile.primary_hdu().unwrap();

            hdu.write_image(&mut fitsfile, &data).unwrap();
            fitsfile.pretty_print().unwrap();

            break;
        }
    }

    //dbg!(IMP_FrameSource_SetFrameDepth(0, 0));
    dbg!(IMP_FrameSource_SetFrameDepth(1, 0));
    //dbg!(IMP_FrameSource_DisableChn(0));
    dbg!(IMP_FrameSource_DisableChn(1));
    dbg!(IMP_FrameSource_DestroyChn(0));
    dbg!(IMP_FrameSource_DestroyChn(1));

    exit_imp(&mut sensor_info);
}

fn main() {
    //println!("{}", opencv::core::get_build_information().unwrap());
    println!("Hello a");
    unsafe {
        imp();
    }
    println!("Hello b");
}

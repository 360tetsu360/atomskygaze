use fitsio::images::{ImageDescription, ImageType};
use fitsio::FitsFile;
use isvp_sys::*;
use opencv::core::*;
use opencv::imgproc::*;
use std::os::raw::c_void;

const SENSOR_NAME: &[u8] = b"gc2053";

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

    let mut chn_attr = IMPFSChnAttr {
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

    dbg!(IMP_FrameSource_CreateChn(0, &mut chn_attr));
    dbg!(IMP_FrameSource_SetChnAttr(0, &chn_attr));
    dbg!(IMP_FrameSource_EnableChn(0));
    dbg!(IMP_FrameSource_SetFrameDepth(0, 4));

    let mut frame_bak: *mut IMPFrameInfo = std::ptr::null_mut();

    for m in 0..50 {
        println!("frame number {}", m);
        dbg!(IMP_FrameSource_GetFrame(0, &mut frame_bak));
        if m == 25 {
            println!(
                "width : {}, height {}",
                (*frame_bak).width,
                (*frame_bak).height
            );
            println!(
                "frame_size : {}, addr {}",
                (*frame_bak).size,
                (*frame_bak).virAddr
            );

            let yuv = Mat::new_rows_cols_with_data_def(1620, 1920, CV_8UC1, (*frame_bak).virAddr as *mut c_void).unwrap();
            let mut dst = Mat::default();
            cvt_color(&yuv, &mut dst, COLOR_YUV2GRAY_NV12, 0).unwrap();

            let data: Vec<u8> = dst
                .to_vec_2d::<u8>()
                .unwrap()
                .iter()
                .flat_map(|v| v.iter())
                .cloned()
                .collect();
            dbg!(data.len());

            let primary_hdu_description = ImageDescription {
                data_type: ImageType::UnsignedByte,
                dimensions: &[1080, 1920],
            };

            let mut fitsfile = FitsFile::create("/media/mmc/test.fits")
                .with_custom_primary(&primary_hdu_description)
                .overwrite()
                .open()
                .unwrap();

            let hdu = fitsfile.primary_hdu().unwrap();

            hdu.write_image(&mut fitsfile, &data).unwrap();
            fitsfile.pretty_print().unwrap();
        }
        dbg!(IMP_FrameSource_ReleaseFrame(0, frame_bak));
    }

    dbg!(IMP_FrameSource_SetFrameDepth(0, 0));
    dbg!(IMP_FrameSource_DisableChn(0));
    dbg!(IMP_FrameSource_DestroyChn(0));

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

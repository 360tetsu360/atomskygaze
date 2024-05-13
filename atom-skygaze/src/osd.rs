use crate::font::BITMAP_ARRAY;
use crate::AppState;
use chrono::*;
use isvp_sys::*;
use log::error;
use std::sync::{Arc, Mutex};

const TEXT_LENGTH: usize = 50;
const VERSION: &str = env!("CARGO_PKG_VERSION");
static mut FONT_HANDLE: i32 = 0;
static mut GRP_NUM: i32 = 0;

unsafe fn imp_osd_init(grp_num: ::std::os::raw::c_int) -> IMPRgnHandle {
    let font_handle: IMPRgnHandle = IMP_OSD_CreateRgn(std::ptr::null_mut());
    if IMP_OSD_RegisterRgn(font_handle, grp_num, std::ptr::null_mut()) < 0 {
        error!("IMP_OSD_RegisterRgn");
        panic!();
    }

    let mut font_attr = IMPOSDRgnAttr {
        type_: IMPOsdRgnType_OSD_REG_BITMAP,
        rect: IMPRect {
            p0: IMPPoint {
                x: 14,
                y: 1080 - 20,
            },
            p1: IMPPoint {
                x: 14 + TEXT_LENGTH as i32 * 14 - 1,
                y: 1080 - 7,
            },
        },
        fmt: IMPPixelFormat_PIX_FMT_MONOWHITE,
        data: IMPOSDRgnAttrData {
            bitmapData: std::ptr::null_mut(),
        },
    };

    if IMP_OSD_SetRgnAttr(font_handle, &mut font_attr) < 0 {
        error!("IMP_OSD_SetRgnAttr");
        panic!();
    }

    let mut gr_font_attr = IMPOSDGrpRgnAttr {
        show: 0,
        offPos: IMPPoint { x: 0, y: 0 },
        scalex: 2.,
        scaley: 2.,
        gAlphaEn: 0,
        fgAlhpa: 0,
        bgAlhpa: 0,
        layer: 0,
    };

    if IMP_OSD_GetGrpRgnAttr(font_handle, grp_num, &mut gr_font_attr) < 0 {
        error!("IMP_OSD_GetGrpRgnAttr");
        panic!();
    }

    gr_font_attr.show = 0;
    gr_font_attr.gAlphaEn = 1;
    gr_font_attr.fgAlhpa = 0xff;
    gr_font_attr.layer = 3;

    gr_font_attr.scalex = 2.;
    gr_font_attr.scaley = 2.;
    gr_font_attr.bgAlhpa = 0;
    gr_font_attr.offPos = IMPPoint { x: 0, y: 0 };

    if IMP_OSD_SetGrpRgnAttr(font_handle, grp_num, &mut gr_font_attr) < 0 {
        error!("IMP_OSD_SetGrpRgnAttr font error !");
        panic!();
    }

    if IMP_OSD_Start(grp_num) < 0 {
        error!("IMP_OSD_Start TimeStamp, Logo, Cover and Rect error !");
        panic!();
    }

    font_handle
}

pub unsafe fn osd_exit() -> bool {
    let mut osdcell = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_OSD,
        groupID: GRP_NUM,
        outputID: 0,
    };

    let mut fscell = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_FS,
        groupID: 0,
        outputID: 0,
    };

    if IMP_System_UnBind(&mut fscell, &mut osdcell) < 0 {
        error!("IMP_System_UnBind error !");
        return false;
    }

    if IMP_OSD_ShowRgn(FONT_HANDLE, GRP_NUM, 0) < 0 {
        error!("IMP_OSD_ShowRgn() timeStamp error");
        return false;
    }

    if IMP_OSD_UnRegisterRgn(FONT_HANDLE, GRP_NUM) < 0 {
        error!("IMP_OSD_UnRegisterRgn() timeStamp error");
        return false;
    }

    IMP_OSD_DestroyRgn(FONT_HANDLE);

    if IMP_OSD_DestroyGroup(GRP_NUM) < 0 {
        error!("IMP_OSD_DestroyGroup() timeStamp error");
        return false;
    }

    true
}

pub unsafe fn imp_osd_bind() -> (::std::os::raw::c_int, IMPRgnHandle) {
    let grp_num = 0;

    if IMP_OSD_CreateGroup(grp_num) < 0 {
        error!("IMP_OSD_CreateGroup error !");
        panic!();
    }

    let font_handle = imp_osd_init(grp_num);

    let mut osdcell = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_OSD,
        groupID: grp_num,
        outputID: 0,
    };

    let mut fscell = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_FS,
        groupID: 0,
        outputID: 0,
    };

    if IMP_System_Bind(&mut fscell, &mut osdcell) < 0 {
        error!("IMP_System_Bind error !");
        panic!();
    }

    FONT_HANDLE = font_handle;
    GRP_NUM = grp_num;

    (grp_num, font_handle)
}

unsafe fn imp_osd_show(
    grp_num: ::std::os::raw::c_int,
    font_handle: IMPRgnHandle,
    show_val: ::std::os::raw::c_int,
) -> bool {
    if IMP_OSD_ShowRgn(font_handle, grp_num, show_val) < 0 {
        error!("IMP_OSD_ShowRgn() timeStamp error");
        return false;
    }

    true
}

pub unsafe fn imp_osd_start(
    grp_num: ::std::os::raw::c_int,
    font_handle: IMPRgnHandle,
    app_state: Arc<Mutex<AppState>>,
    flag: Arc<Mutex<bool>>,
) {
    let mut timestamp_data = vec![0u8; TEXT_LENGTH * 14 * 14];
    let mut last_state = true;

    if !imp_osd_show(grp_num, font_handle, 1) {
        error!("OSD show error");
        return;
    }

    loop {
        let shutdown_flag = match flag.lock() {
            Ok(guard) => guard,
            Err(_) => continue,
        };

        if *shutdown_flag {
            break;
        }
        drop(shutdown_flag);

        let app_state_tmp = match app_state.lock() {
            Ok(guard) => guard,
            Err(_) => continue,
        };
        if last_state != app_state_tmp.timestamp {
            if app_state_tmp.timestamp {
                if !imp_osd_show(grp_num, font_handle, 1) {
                    error!("OSD show error");
                    return;
                }
            } else if !imp_osd_show(grp_num, font_handle, 0) {
                error!("OSD show error");
                return;
            }
        }
        last_state = app_state_tmp.timestamp;

        if app_state_tmp.timestamp {
            timestamp_data.fill(0);
            let now: DateTime<Utc> = Utc::now();
            let offset = if app_state_tmp.timezone < 0 {
                FixedOffset::west_opt(-app_state_tmp.timezone)
            } else {
                FixedOffset::east_opt(app_state_tmp.timezone)
            }
            .unwrap();
            let time: DateTime<FixedOffset> = now.with_timezone(&offset);
            let fractional_second = (time.timestamp_subsec_millis() as f64) / 100.0;
            let text = format!(
                "{}.{} {}   ATOM-SKYGAZE v{}",
                time.format("%Y-%m-%d %H:%M:%S"),
                fractional_second as i32,
                time.format("%:z"),
                VERSION
            );

            for (i, c) in text.chars().enumerate() {
                let char_index = match c {
                    '.' => 0,
                    '-' => 49,
                    '+' => 50,
                    ':' => 12,
                    '0'..='9' => 2 + (c as usize - '0' as usize),
                    'A'..='Z' => 19 + (c as usize - 'A' as usize),
                    'v' => 51,
                    ' ' => continue,
                    _ => continue,
                };

                let base = i * 14;
                for j in 0..14 {
                    let char_line = &BITMAP_ARRAY
                        [char_index * 14 * 12 + j * 12..char_index * 14 * 12 + j * 12 + 12];
                    timestamp_data
                        [j * 14 * TEXT_LENGTH + base + 1..j * 14 * TEXT_LENGTH + base + 13]
                        .copy_from_slice(char_line);
                }
            }

            let mut data = IMPOSDRgnAttrData {
                bitmapData: timestamp_data.as_mut_ptr() as *mut ::std::os::raw::c_void,
            };
            IMP_OSD_UpdateRgnAttrData(font_handle, &mut data);
        }
        drop(app_state_tmp);

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

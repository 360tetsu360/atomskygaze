use crate::font::*;
use crate::AppState;
use chrono::*;
use isvp_sys::*;
use log::{error, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const TEXT_LENGTH: usize = 43 + ver_len(VERSION);
const FONT_SCALE: f32 = 1.;
static mut FONT_HANDLE: i32 = 0;
static mut GRP_NUM: i32 = 0;

const fn ver_len(ver: &str) -> usize {
    ver.len()
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum TimestampPos {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

unsafe fn imp_osd_init(grp_num: ::std::os::raw::c_int) -> IMPRgnHandle {
    let font_handle: IMPRgnHandle = IMP_OSD_CreateRgn(std::ptr::null_mut());
    if IMP_OSD_RegisterRgn(font_handle, grp_num, std::ptr::null_mut()) < 0 {
        error!("IMP_OSD_RegisterRgn failed");
        panic!();
    }

    let mut font_attr = IMPOSDRgnAttr {
        type_: IMPOsdRgnType_OSD_REG_BITMAP,
        rect: IMPRect {
            p0: IMPPoint {
                x: (CHAR_WIDTH + 2) as i32,
                y: 1080 - 21,
            },
            p1: IMPPoint {
                x: ((CHAR_WIDTH + 2) + TEXT_LENGTH * (CHAR_WIDTH + 2) - 1) as i32,
                y: 1080 - 6,
            },
        },
        fmt: IMPPixelFormat_PIX_FMT_MONOWHITE,
        data: IMPOSDRgnAttrData {
            bitmapData: std::ptr::null_mut(),
        },
    };

    if IMP_OSD_SetRgnAttr(font_handle, &mut font_attr) < 0 {
        error!("IMP_OSD_SetRgnAttr failed");
        panic!();
    }

    let mut gr_font_attr = IMPOSDGrpRgnAttr {
        show: 0,
        offPos: IMPPoint { x: 0, y: 0 },
        scalex: FONT_SCALE,
        scaley: FONT_SCALE,
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

    gr_font_attr.scalex = FONT_SCALE;
    gr_font_attr.scaley = FONT_SCALE;
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
        panic!();
    }

    if IMP_OSD_ShowRgn(FONT_HANDLE, GRP_NUM, 0) < 0 {
        error!("IMP_OSD_ShowRgn() timeStamp error");
        panic!();
    }

    if IMP_OSD_UnRegisterRgn(FONT_HANDLE, GRP_NUM) < 0 {
        error!("IMP_OSD_UnRegisterRgn() timeStamp error");
        panic!();
    }

    IMP_OSD_DestroyRgn(FONT_HANDLE);

    if IMP_OSD_DestroyGroup(GRP_NUM) < 0 {
        error!("IMP_OSD_DestroyGroup() timeStamp error");
        panic!();
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
) {
    if IMP_OSD_ShowRgn(font_handle, grp_num, show_val) < 0 {
        error!("IMP_OSD_ShowRgn() timeStamp error");
        panic!();
    }
}

pub unsafe fn imp_osd_start(
    grp_num: ::std::os::raw::c_int,
    font_handle: IMPRgnHandle,
    app_state: Arc<Mutex<AppState>>,
    flag: Arc<AtomicBool>,
) {
    let mut timestamp_data = vec![0u8; TEXT_LENGTH * (CHAR_WIDTH + 2) * CHAR_HEIGHT];
    let mut last_state = true;
    let mut last_ts_pos = TimestampPos::BottomLeft;
    let uppercase_ver = VERSION.to_uppercase();

    imp_osd_show(grp_num, font_handle, 1);

    loop {
        if flag.load(Ordering::Relaxed) {
            log::info!("Stopping osd_loop");
            break;
        }

        let mut app_state_tmp = match app_state.lock() {
            Ok(guard) => guard,
            Err(e) => {
                warn!(
                    "app_state mutex lock error : {} at{}:{}",
                    e,
                    file!(),
                    line!()
                );
                continue;
            }
        };
        if last_state != app_state_tmp.timestamp {
            if app_state_tmp.timestamp {
                imp_osd_show(grp_num, font_handle, 1);
            } else {
                imp_osd_show(grp_num, font_handle, 0);
            }
        }
        last_state = app_state_tmp.timestamp;

        if last_ts_pos as u32 != app_state_tmp.timestamp_pos {
            let mut rect = IMPRect {
                p0: IMPPoint { x: 0, y: 0 },
                p1: IMPPoint { x: 0, y: 0 },
            };
            match app_state_tmp.timestamp_pos {
                0 => {
                    rect.p0.x = (CHAR_WIDTH + 2) as i32;
                    rect.p0.y = 5;
                    rect.p1.x = ((CHAR_WIDTH + 2) + TEXT_LENGTH * (CHAR_WIDTH + 2) - 1) as i32;
                    rect.p1.y = 20;
                    last_ts_pos = TimestampPos::TopLeft;
                }
                1 => {
                    rect.p0.x = 1920 - ((TEXT_LENGTH + 1) * (CHAR_WIDTH + 2)) as i32;
                    rect.p0.y = 5;
                    rect.p1.x = 1920 - (CHAR_WIDTH + 3) as i32;
                    rect.p1.y = 20;
                    last_ts_pos = TimestampPos::TopRight;
                }
                2 => {
                    rect.p0.x = (CHAR_WIDTH + 2) as i32;
                    rect.p0.y = 1080 - 21;
                    rect.p1.x = ((CHAR_WIDTH + 2) + TEXT_LENGTH * (CHAR_WIDTH + 2) - 1) as i32;
                    rect.p1.y = 1080 - 6;
                    last_ts_pos = TimestampPos::BottomLeft;
                }
                3 => {
                    rect.p0.x = 1920 - ((TEXT_LENGTH + 1) * (CHAR_WIDTH + 2)) as i32;
                    rect.p0.y = 1080 - 21;
                    rect.p1.x = 1920 - (CHAR_WIDTH + 3) as i32;
                    rect.p1.y = 1080 - 6;
                    last_ts_pos = TimestampPos::BottomRight;
                }
                _ => {
                    warn!(
                        "Unknown TimestampPos type : {}",
                        app_state_tmp.timestamp_pos
                    );
                    app_state_tmp.timestamp_pos = last_ts_pos as u32;
                    continue;
                }
            }

            let mut font_attr = IMPOSDRgnAttr {
                type_: IMPOsdRgnType_OSD_REG_BITMAP,
                rect,
                fmt: IMPPixelFormat_PIX_FMT_MONOWHITE,
                data: IMPOSDRgnAttrData {
                    bitmapData: timestamp_data.as_mut_ptr() as *mut ::std::os::raw::c_void,
                },
            };

            if IMP_OSD_SetRgnAttr(font_handle, &mut font_attr) < 0 {
                error!("Failed IMP_OSD_SetRgnAttr");
                panic!();
            }
        }

        if app_state_tmp.timestamp {
            timestamp_data.fill(0);
            let now: DateTime<Utc> = Utc::now();
            let offset = FixedOffset::east_opt(app_state_tmp.timezone).unwrap();
            drop(app_state_tmp);

            let time: DateTime<FixedOffset> = now.with_timezone(&offset);
            let fractional_second = (time.timestamp_subsec_millis() as f64) / 100.0;
            let text = format!(
                "{}.{} {}   ATOM-SKYGAZE v{}",
                time.format("%Y-%m-%d %H:%M:%S"),
                fractional_second as i32,
                time.format("%:z"),
                uppercase_ver
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
                    _ => {
                        warn!("{} is not in the bitmap", c);
                        continue;
                    }
                };

                let base_offset = i * (CHAR_WIDTH + 1);
                for j in 0..CHAR_HEIGHT {
                    let char_line = &BITMAP_ARRAY[(char_index * CHAR_HEIGHT + j) * CHAR_WIDTH
                        ..(char_index * CHAR_HEIGHT + j + 1) * CHAR_WIDTH];
                    timestamp_data[j * (CHAR_WIDTH + 2) * TEXT_LENGTH + base_offset + 1
                        ..j * (CHAR_WIDTH + 2) * TEXT_LENGTH + base_offset + (CHAR_WIDTH + 1)]
                        .copy_from_slice(char_line);
                }
            }

            let mut data = IMPOSDRgnAttrData {
                bitmapData: timestamp_data.as_mut_ptr() as *mut ::std::os::raw::c_void,
            };
            IMP_OSD_UpdateRgnAttrData(font_handle, &mut data);
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

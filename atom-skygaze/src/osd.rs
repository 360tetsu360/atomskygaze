use isvp_sys::*;
use log::{error, info};

pub unsafe fn init(grp_num: ::str::os::raw::c_int) -> IMPRgnHandle {
    let mut font_handle: IMPRgnHandle = IMP_OSD_CreateRgn(0);
    if IMP_OSD_RegisterRgn(pr_handle, grp_num, 0) < 0 {
        error!("IMP_OSD_RegisterRgn");
        panic!();
    }

    let mut font_attr = IMPOSDRgnAttr {
        type_: IMPOsdRgnType_OSD_REG_PIC,
        rect: IMPREct {
            p0: IMPPoint {
                x: 10,
                y: 10,
            },
            p1: IMPPoint {
                x: 10 + 20 * 16 -1,
                y: 10 + 34 - 1,
            },
        },
        fmt: IMPPixelFormat_PIX_FMT_BGRA,
        data: IMPOSDRgnAttrData {
            picData: picData {
                pdata: std::ptr::null_mut(),
            },
        },
    };

    if IMP_OSD_SetRgnAttr(font_handle, &mut font_attr) < 0 {
        error!("IMP_OSD_SetRgnAttr");
        panic!();
    }

    let mut gr_font_attr = IMPOSDGrpRgnAttr {
        show: 0,
        offPos: IMPPoint {
            x: 0,
            y: 0,
        },
        scalex: 0.,
        scaley: 0.,
        gAlphaEn: 0,
        fgAlhpa: 0,
        bgAlhpa: 0,
        layer: 0
    };

    if IMP_OSD_GetGrpRgnAttr(font_handle, grp_num, &mut gr_font_attr) < 0 {
        error!("IMP_OSD_GetGrpRgnAttr");
        panic!();
    }

    gr_font_attr.show = 0;
	gr_font_attr.gAlphaEn = 1;
	gr_font_attr.fgAlhpa = 0xff;
	gr_font_attr.layer = 3;

	gr_font_attr.scalex = 0.;
	gr_font_attr.scaley = 0.;
	gr_font_attr.bgAlhpa = 0;
	gr_font_attr.offPos = IMPPoint {
        x: 0,
        y: 0,
    };

    if IMP_OSD_SetGrpRgnAttr(font_handle, grp_num, &mut gr_font_attr) < 0 {
		error!("IMP_OSD_SetGrpRgnAttr font error !");
		panic!();
	}

	if IMP_OSD_Start(grp_num) < 0 {
		error!("IMP_OSD_Start TimeStamp, Logo, Cover and Rect error !");
		panic!();
	}

    return font_handle;
}

pub unsafe fn bind() {
    let grp_num = 0;

    let font_handle = init(grp_num);
    
    let mut osdcell = IMPCell {
        deviceID: DEV_ID_OSD,
        groupID: grp_num, 
        outputID: 0
    };

    let mut fscell = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_FS,
        groupID: 0,
        outputID: 0
    };

    let mut enccell IMPCell {
        deviceID: IMPDeviceID_DEV_ID_ENC,
        groupID: 0,
        outputID: 0
    };

    if IMP_System_Bind(&mut fscell, &mut osdcell) < 0 {
        error!("IMP_System_Bind error !");
		panic!();
    }

    if IMP_System_Bind(&mut fscell, &mut enccell) < 0 {
        error!("IMP_System_Bind error !");
		panic!();
    }

    let timestamp_data = Vec::<u32>::with_capacity(20 * 34 * 16);
}

unsafe fn osd_show(grp_num: ::str::os::raw::c_int, mut handle: IMPOSDRgnAttr){
	if IMP_OSD_ShowRgn(&mut handle, 0, 1) {
		IMP_LOG_ERR(TAG, "IMP_OSD_ShowRgn() timeStamp error\n");
		return -1;
	}
}

use crate::osd;
use crate::AppState;
use isvp_sys::*;
use log::error;
use std::ptr::addr_of_mut;

const SENSOR_NAME: &[u8] = b"gc2053";
const BITRATE_720P_KBS: u32 = 500;
const SENSOR_WIDTH: i32 = 1920;
const SENSOR_HEIGHT: i32 = 1080;

static mut SENSOR_INFO: IMPSensorInfo = IMPSensorInfo {
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

static mut CHANNEL_ATTRIBUTES: [IMPFSChnAttr; 2] = [
    IMPFSChnAttr {
        picWidth: SENSOR_WIDTH,
        picHeight: SENSOR_HEIGHT,
        pixFmt: IMPPixelFormat_PIX_FMT_NV12,
        crop: IMPFSChnCrop {
            enable: 1,
            left: 0,
            top: 0,
            width: SENSOR_WIDTH,
            height: SENSOR_HEIGHT,
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
    },
    IMPFSChnAttr {
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
    },
];

pub unsafe fn imp_init(app_state: AppState) -> bool {
    IMP_OSD_SetPoolSize(512 * 1024);

    SENSOR_INFO.name[..SENSOR_NAME.len()]
        .copy_from_slice(std::mem::transmute::<&[u8], &[i8]>(SENSOR_NAME));
    SENSOR_INFO.__bindgen_anon_1.i2c.type_[..SENSOR_NAME.len()]
        .copy_from_slice(std::mem::transmute::<&[u8], &[i8]>(SENSOR_NAME));

    if IMP_ISP_Open() < 0 {
        error!("IMP_ISP_Open failed");
        return false;
    }

    if IMP_ISP_AddSensor(addr_of_mut!(SENSOR_INFO)) < 0 {
        error!("IMP_ISP_AddSensor failed");
        return false;
    }

    if IMP_ISP_EnableSensor() < 0 {
        error!("IMP_ISP_EnableSensor failed");
        return false;
    }

    if IMP_System_Init() < 0 {
        error!("IMP_System_Init failed");
        return false;
    }

    if IMP_ISP_EnableTuning() < 0 {
        error!("IMP_ISP_EnableTuning failed");
        return false;
    }

    if IMP_ISP_Tuning_SetContrast(app_state.contrast) < 0 {
        error!("IMP_ISP_Tuning_SetContrast failed");
        return false;
    }

    if IMP_ISP_Tuning_SetSharpness(app_state.sharpness) < 0 {
        error!("IMP_ISP_Tuning_SetSharpness failed");
        return false;
    }

    if IMP_ISP_Tuning_SetSaturation(app_state.saturation) < 0 {
        error!("IMP_ISP_SetSaturation failed");
        return false;
    }

    if IMP_ISP_Tuning_SetBrightness(app_state.brightness) < 0 {
        error!("IMP_ISP_SetBrightness failed");
        return false;
    }

    if IMP_ISP_Tuning_SetAntiFlickerAttr(IMPISPAntiflickerAttr_IMPISP_ANTIFLICKER_DISABLE) < 0 {
        error!("IMP_ISP_Tuning_SetAntiFlickerAttr failed");
        return false;
    }

    let mode = if app_state.night_mode {
        IMPISPRunningMode_IMPISP_RUNNING_MODE_NIGHT
    } else {
        IMPISPRunningMode_IMPISP_RUNNING_MODE_DAY
    };

    if IMP_ISP_Tuning_SetISPRunningMode(mode) < 0 {
        error!("IMP_ISP_Tuning_SetTSPRunningMode failed");
        return false;
    }

    if app_state.flip.0 {
        IMP_ISP_Tuning_SetISPHflip(IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE);
    } else {
        IMP_ISP_Tuning_SetISPHflip(IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE);
    }

    if app_state.flip.1 {
        IMP_ISP_Tuning_SetISPVflip(IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE);
    } else {
        IMP_ISP_Tuning_SetISPVflip(IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE);
    }

    IMP_ISP_Tuning_SetSensorFPS(app_state.fps, 1);

    true
}

pub unsafe fn imp_shutdown() -> bool {
    imp_framesource_stop();
    osd::osd_exit();
    imp_encoder_exit();
    imp_framesource_exit();
    imp_exit();
    true
}

pub unsafe fn imp_exit() -> bool {
    if IMP_System_Exit() < 0 {
        error!("IMP_System_Exit failed");
        return false;
    }

    if IMP_ISP_DisableSensor() < 0 {
        error!("IMP_ISP_DisableSensor failed");
        return false;
    }

    if IMP_ISP_DelSensor(addr_of_mut!(SENSOR_INFO)) < 0 {
        error!("IMP_ISP_DelSensor failed");
        return false;
    }

    if IMP_ISP_DisableTuning() < 0 {
        error!("IMP_ISP_DisableTuning failed");
        return false;
    }

    if IMP_ISP_Close() < 0 {
        error!("IMP_ISP_Close failed");
        return false;
    }

    true
}

pub unsafe fn imp_framesource_init() -> bool {
    if IMP_FrameSource_CreateChn(0, addr_of_mut!(CHANNEL_ATTRIBUTES[0])) < 0 {
        error!("IMP_FrameSource_CreateChn failed");
        return false;
    }

    if IMP_FrameSource_SetChnAttr(0, &CHANNEL_ATTRIBUTES[0]) < 0 {
        error!("IMP_FrameSource_SetChnAttr failed");
        return false;
    }

    if IMP_FrameSource_CreateChn(1, addr_of_mut!(CHANNEL_ATTRIBUTES[1])) < 0 {
        error!("IMP_FrameSource_CreateChn failed");
        return false;
    }

    if IMP_FrameSource_SetChnAttr(1, &CHANNEL_ATTRIBUTES[1]) < 0 {
        error!("IMP_FrameSource_SetChnAttr failed");
        return false;
    }

    true
}

pub unsafe fn imp_framesource_start() -> bool {
    if IMP_FrameSource_EnableChn(0) < 0 {
        error!("IMP_FrameSource_EnableChn failed");
        return false;
    }

    if IMP_FrameSource_EnableChn(1) < 0 {
        error!("IMP_FrameSource_EnableChn failed");
        return false;
    }

    true
}

pub unsafe fn imp_framesource_stop() -> bool {
    if IMP_FrameSource_DisableChn(0) < 0 {
        error!("IMP_FrameSource_DisableChn failed");
        return false;
    }

    if IMP_FrameSource_DisableChn(1) < 0 {
        error!("IMP_FrameSource_DisableChn failed");
        return false;
    }

    true
}

pub unsafe fn imp_framesource_exit() -> bool {
    if IMP_FrameSource_DestroyChn(0) < 0 {
        error!("IMP_FrameSource_DestroyChn failed");
        return false;
    }

    if IMP_FrameSource_DestroyChn(1) < 0 {
        error!("IMP_FrameSource_DestroyChn failed");
        return false;
    }

    true
}

pub unsafe fn imp_encoder_init() -> bool {
    if IMP_Encoder_CreateGroup(0) < 0 {
        error!("IMP_Encoder_CreateGroup failed");
        return false;
    }

    let bitrate: f32 = 2000.0 * (1920. * 1080.) / (1280. * 720.);
    let mut encoder_attr = IMPEncoderChnAttr {
        encAttr: IMPEncoderEncAttr {
            eProfile: 0,
            uLevel: 0,
            uTier: 0,
            uWidth: 0,
            uHeight: 0,
            ePicFormat: 0,
            eEncOptions: 0,
            eEncTools: 0,
            crop: IMPEncoderCropCfg {
                enable: false,
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
        },
        rcAttr: IMPEncoderRcAttr {
            attrRcMode: IMPEncoderAttrRcMode {
                rcMode: 0,
                __bindgen_anon_1: IMPEncoderAttrRcMode__bindgen_ty_1 {
                    attrFixQp: IMPEncoderAttrFixQP { iInitialQP: 0 },
                },
            },
            outFrmRate: IMPEncoderFrmRate {
                frmRateNum: 0,
                frmRateDen: 0,
            },
        },
        gopAttr: IMPEncoderGopAttr {
            uGopCtrlMode: 0,
            uGopLength: 0,
            uNotifyUserLTInter: 0,
            uMaxSameSenceCnt: 0,
            bEnableLT: false,
            uFreqLT: 0,
            bLTRC: false,
        },
    };

    if IMP_Encoder_SetDefaultParam(
        &mut encoder_attr,
        IMPEncoderProfile_IMP_ENC_PROFILE_HEVC_MAIN,
        IMPEncoderRcMode_IMP_ENC_RC_MODE_VBR,
        SENSOR_WIDTH as u16,
        SENSOR_HEIGHT as u16,
        25,
        1,
        50,
        2,
        -1,
        bitrate.round() as u32,
    ) < 0
    {
        error!("IMP_Encoder_SetDefaultParam failed");
        return false;
    }

    if IMP_Encoder_CreateChn(0, &encoder_attr) < 0 {
        error!("IMP_Encoder_CreateChn failed");
        return false;
    }

    if IMP_Encoder_RegisterChn(0, 0) < 0 {
        error!("IMP_Encoder_CreateChn failed");
        return false;
    }

    if IMP_Encoder_CreateGroup(1) < 0 {
        error!("IMP_Encoder_CreateGroup failed");
        return false;
    }

    let ratio = 1.0 / (f32::log10((1280. * 720.) / (640. * 360.)) + 1.0);
    let bitrate = (BITRATE_720P_KBS as f32 * ratio) as u32;

    let mut encoder_attr = IMPEncoderChnAttr {
        encAttr: IMPEncoderEncAttr {
            eProfile: 0,
            uLevel: 0,
            uTier: 0,
            uWidth: 0,
            uHeight: 0,
            ePicFormat: 0,
            eEncOptions: 0,
            eEncTools: 0,
            crop: IMPEncoderCropCfg {
                enable: false,
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
        },
        rcAttr: IMPEncoderRcAttr {
            attrRcMode: IMPEncoderAttrRcMode {
                rcMode: 0,
                __bindgen_anon_1: IMPEncoderAttrRcMode__bindgen_ty_1 {
                    attrFixQp: IMPEncoderAttrFixQP { iInitialQP: 0 },
                },
            },
            outFrmRate: IMPEncoderFrmRate {
                frmRateNum: 0,
                frmRateDen: 0,
            },
        },
        gopAttr: IMPEncoderGopAttr {
            uGopCtrlMode: 0,
            uGopLength: 0,
            uNotifyUserLTInter: 0,
            uMaxSameSenceCnt: 0,
            bEnableLT: false,
            uFreqLT: 0,
            bLTRC: false,
        },
    };

    if IMP_Encoder_SetDefaultParam(
        &mut encoder_attr,
        IMPEncoderProfile_IMP_ENC_PROFILE_HEVC_MAIN,
        IMPEncoderRcMode_IMP_ENC_RC_MODE_CAPPED_QUALITY,
        640,
        360,
        25,
        1,
        25 * 2,
        2,
        -1,
        bitrate,
    ) < 0
    {
        error!("IMP_Encoder_SetDefaultParam failed");
        return false;
    }

    if IMP_Encoder_CreateChn(1, &encoder_attr) < 0 {
        error!("IMP_Encoder_CreateChn failed");
        return false;
    }

    if IMP_Encoder_RegisterChn(1, 1) < 0 {
        error!("IMP_Encoder_CreateChn failed");
        return false;
    }

    true
}

pub unsafe fn imp_encoder_exit() -> bool {
    let mut framesource_chn = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_FS,
        groupID: 0,
        outputID: 0,
    };

    let mut imp_encoder = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_ENC,
        groupID: 0,
        outputID: 0,
    };

    if IMP_System_UnBind(&mut framesource_chn, &mut imp_encoder) < 0 {
        error!("hevc IMP_System_UnBind failed");
        return false;
    }

    let mut framesource_chn = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_FS,
        groupID: 1,
        outputID: 0,
    };

    let mut imp_encoder = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_ENC,
        groupID: 1,
        outputID: 0,
    };

    if IMP_System_UnBind(&mut framesource_chn, &mut imp_encoder) < 0 {
        error!("jpeg IMP_System_UnBind failed");
        return false;
    }

    if IMP_Encoder_UnRegisterChn(0) < 0 {
        error!("chn0 IMP_Encoder_UnRegisterChn failed");
        return false;
    }

    if IMP_Encoder_DestroyChn(0) < 0 {
        error!("chn0 IMP_Encoder_DestroyChn failed");
        return false;
    }

    if IMP_Encoder_UnRegisterChn(1) < 0 {
        error!("chn0 IMP_Encoder_UnRegisterChn failed");
        return false;
    }

    if IMP_Encoder_DestroyChn(1) < 0 {
        error!("chn0 IMP_Encoder_DestroyChn failed");
        return false;
    }

    true
}

pub unsafe fn imp_jpeg_init() -> bool {
    let mut channel_attr = IMPEncoderChnAttr {
        encAttr: IMPEncoderEncAttr {
            eProfile: 0,
            uLevel: 0,
            uTier: 0,
            uWidth: 0,
            uHeight: 0,
            ePicFormat: 0,
            eEncOptions: 0,
            eEncTools: 0,
            crop: IMPEncoderCropCfg {
                enable: false,
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
        },
        rcAttr: IMPEncoderRcAttr {
            attrRcMode: IMPEncoderAttrRcMode {
                rcMode: 0,
                __bindgen_anon_1: IMPEncoderAttrRcMode__bindgen_ty_1 {
                    attrFixQp: IMPEncoderAttrFixQP { iInitialQP: 0 },
                },
            },
            outFrmRate: IMPEncoderFrmRate {
                frmRateNum: 0,
                frmRateDen: 0,
            },
        },
        gopAttr: IMPEncoderGopAttr {
            uGopCtrlMode: 0,
            uGopLength: 0,
            uNotifyUserLTInter: 0,
            uMaxSameSenceCnt: 0,
            bEnableLT: false,
            uFreqLT: 0,
            bLTRC: false,
        },
    };

    if IMP_Encoder_SetDefaultParam(
        &mut channel_attr,
        IMPEncoderProfile_IMP_ENC_PROFILE_JPEG,
        IMPEncoderRcMode_IMP_ENC_RC_MODE_FIXQP,
        640,
        360,
        25,
        1,
        0,
        0,
        25,
        0,
    ) < 0
    {
        error!("IMP_Encoder_SetDefaultParam failed");
        return false;
    }

    if IMP_Encoder_CreateChn(2, &channel_attr) < 0 {
        error!("IMP_Encoder_CreateChn failed");
        return false;
    }

    if IMP_Encoder_RegisterChn(1, 2) < 0 {
        error!("IMP_Encoder_RegisterChn failed");
        return false;
    }

    let mut framesource_chn = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_FS,
        groupID: 1,
        outputID: 0,
    };

    let mut imp_encoder = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_ENC,
        groupID: 1,
        outputID: 0,
    };

    if IMP_System_Bind(&mut framesource_chn, &mut imp_encoder) < 0 {
        error!("IMP_System_Bind failed");
        return false;
    }

    true
}

pub unsafe fn imp_hevc_init() -> bool {
    let mut channel_attr = IMPEncoderChnAttr {
        encAttr: IMPEncoderEncAttr {
            eProfile: 0,
            uLevel: 0,
            uTier: 0,
            uWidth: 0,
            uHeight: 0,
            ePicFormat: 0,
            eEncOptions: 0,
            eEncTools: 0,
            crop: IMPEncoderCropCfg {
                enable: false,
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
        },
        rcAttr: IMPEncoderRcAttr {
            attrRcMode: IMPEncoderAttrRcMode {
                rcMode: 0,
                __bindgen_anon_1: IMPEncoderAttrRcMode__bindgen_ty_1 {
                    attrFixQp: IMPEncoderAttrFixQP { iInitialQP: 0 },
                },
            },
            outFrmRate: IMPEncoderFrmRate {
                frmRateNum: 0,
                frmRateDen: 0,
            },
        },
        gopAttr: IMPEncoderGopAttr {
            uGopCtrlMode: 0,
            uGopLength: 0,
            uNotifyUserLTInter: 0,
            uMaxSameSenceCnt: 0,
            bEnableLT: false,
            uFreqLT: 0,
            bLTRC: false,
        },
    };

    let ratio = 1.0 / (f32::log10((1920. * 1080.) / (640. * 360.)) + 1.0);
    let bitrate = (BITRATE_720P_KBS as f32 * ratio) as u32;

    if IMP_Encoder_SetDefaultParam(
        &mut channel_attr,
        IMPEncoderProfile_IMP_ENC_PROFILE_HEVC_MAIN,
        IMPEncoderRcMode_IMP_ENC_RC_MODE_FIXQP,
        SENSOR_WIDTH as u16,
        SENSOR_HEIGHT as u16,
        25,
        1,
        50,
        2,
        38,
        bitrate,
    ) < 0
    {
        error!("IMP_Encoder_SetDefaultParam failed");
        return false;
    }

    if IMP_Encoder_CreateChn(3, &channel_attr) < 0 {
        error!("IMP_Encoder_CreateChn failed");
        return false;
    }

    if IMP_Encoder_RegisterChn(0, 3) < 0 {
        error!("IMP_Encoder_RegisterChn failed");
        return false;
    }

    let mut framesource_chn = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_FS,
        groupID: 0,
        outputID: 0,
    };

    let mut imp_encoder = IMPCell {
        deviceID: IMPDeviceID_DEV_ID_ENC,
        groupID: 0,
        outputID: 0,
    };

    if IMP_System_Bind(&mut framesource_chn, &mut imp_encoder) < 0 {
        error!("IMP_System_Bind failed");
        return false;
    }

    true
}

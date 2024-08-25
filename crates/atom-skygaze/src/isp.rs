use isvp_sys::*;

pub unsafe fn log_all_value() {
    let mut v = 0;
    IMP_ISP_Tuning_GetAntiFlickerAttr(&mut v);
    log::debug!("anti flicker {}", v);
    let mut vu8 = 0;
    IMP_ISP_Tuning_GetBrightness(&mut vu8);
    log::debug!("brightness {}", vu8);
    IMP_ISP_Tuning_GetContrast(&mut vu8);
    log::debug!("contrast {}", vu8);
    IMP_ISP_Tuning_GetSharpness(&mut vu8);
    log::debug!("sharpness {}", vu8);
    IMP_ISP_Tuning_GetSaturation(&mut vu8);
    log::debug!("saturation {}", vu8);
    IMP_ISP_Tuning_GetTotalGain(&mut v);
    log::debug!("total gain {}", v);
    IMP_ISP_Tuning_GetISPHflip(&mut v);
    log::debug!("hflip {}", v);
    IMP_ISP_Tuning_GetISPVflip(&mut v);
    log::debug!("vflip {}", v);
    IMP_ISP_Tuning_GetISPRunningMode(&mut v);
    log::debug!("running mode {}", v);

    let mut vi = 0;
    IMP_ISP_Tuning_GetAeComp(&mut vi);
    log::debug!("ae comp {}", vi);
    IMP_ISP_Tuning_GetAeLuma(&mut vi);
    log::debug!("ae luma {}", v);

    let mut vexpr = IMPISPExpr {
        s_attr: isp_core_expr_attr__bindgen_ty_1 {
            mode: 0,
            unit: 0,
            time: 0,
        },
    };

    IMP_ISP_Tuning_GetExpr(&mut vexpr);
    log::debug!("g_attr mode {}", vexpr.g_attr.mode);
    log::debug!("g_attr time {}", vexpr.g_attr.integration_time);
    log::debug!("g_attr tmin {}", vexpr.g_attr.integration_time_min);
    log::debug!("g_attr tmax {}", vexpr.g_attr.integration_time_max);
    log::debug!("g_attr inus {}", vexpr.g_attr.one_line_expr_in_us);

    let mut vwb = IMPISPWB {
        mode: 0,
        rgain: 0,
        bgain: 0,
    };
    IMP_ISP_Tuning_GetWB(&mut vwb);
    log::debug!("wbmode {}", vwb.mode);
    log::debug!("wb rgain {}", vwb.rgain);
    log::debug!("wb bgain {}", vwb.bgain);

    IMP_ISP_Tuning_GetWB_Statis(&mut vwb);
    log::debug!("wbst mode {}", vwb.mode);
    log::debug!("wbst rgain {}", vwb.rgain);
    log::debug!("wbst bgain {}", vwb.bgain);

    IMP_ISP_Tuning_GetWB_GOL_Statis(&mut vwb);
    log::debug!("wbgolst mode {}", vwb.mode);
    log::debug!("wbgolst rgain {}", vwb.rgain);
    log::debug!("wbgolst bgain {}", vwb.bgain);

    let mut vrgb = IMPISPCOEFFTWB {
        rgb_coefft_wb_r: 0,
        rgb_coefft_wb_g: 0,
        rgb_coefft_wb_b: 0,
    };
    IMP_ISP_Tuning_Awb_GetRgbCoefft(&mut vrgb);
    log::debug!("rgb r {}", vrgb.rgb_coefft_wb_r);
    log::debug!("rgb g {}", vrgb.rgb_coefft_wb_g);
    log::debug!("rgb b {}", vrgb.rgb_coefft_wb_b);

    let mut vu = 0;
    IMP_ISP_Tuning_GetMaxAgain(&mut vu);
    log::debug!("max again {}", vu);
    IMP_ISP_Tuning_GetMaxDgain(&mut vu);
    log::debug!("max dgain {}", vu);
    IMP_ISP_Tuning_GetHiLightDepress(&mut vu);
    log::debug!("hi light depress {}", vu);

    let mut eva_attr = IMPISPEVAttr {
        ev: 0,
        expr_us: 0,
        ev_log2: 0,
        again: 0,
        dgain: 0,
        gain_log2: 0,
    };

    IMP_ISP_Tuning_GetEVAttr(&mut eva_attr);

    log::debug!("ev {}", eva_attr.dgain);
    log::debug!("expr_us {}", eva_attr.dgain);
    log::debug!("ev_log2 {}", eva_attr.dgain);
    log::debug!("again {}", eva_attr.dgain);
    log::debug!("dgain {}", eva_attr.dgain);
    log::debug!("gain_log2 {}", eva_attr.gain_log2);
}

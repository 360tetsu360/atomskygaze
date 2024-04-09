use isvp_sys::*;

pub unsafe fn log_all_value() {
    let mut v = 0;
    IMP_ISP_Tuning_GetAntiFlickerAttr(&mut v);
    println!("anti flicker {}", v);
    let mut vu8 = 0;
    IMP_ISP_Tuning_GetBrightness(&mut vu8);
    println!("brightness {}", vu8);
    IMP_ISP_Tuning_GetContrast(&mut vu8);
    println!("contrast {}", vu8);
    IMP_ISP_Tuning_GetSharpness(&mut vu8);
    println!("sharpness {}", vu8);
    IMP_ISP_Tuning_GetSaturation(&mut vu8);
    println!("saturation {}", vu8);
    IMP_ISP_Tuning_GetTotalGain(&mut v);
    println!("total gain {}", v);
    IMP_ISP_Tuning_GetISPHflip(&mut v);
    println!("hflip {}", v);
    IMP_ISP_Tuning_GetISPVflip(&mut v);
    println!("vflip {}", v);
    IMP_ISP_Tuning_GetISPRunningMode(&mut v);
    println!("running mode {}", v);
    //IMP_ISP_Tuning_GetGamma(&mut v);
    //println!("gamma {}", v);
    let mut vi = 0;
    IMP_ISP_Tuning_GetAeComp(&mut vi);
    println!("ae comp {}", vi);
    IMP_ISP_Tuning_GetAeLuma(&mut vi);
    println!("ae luma {}", v);

    let mut vexpr = IMPISPExpr {
        s_attr: isp_core_expr_attr__bindgen_ty_1 {
            mode: 0,
            unit: 0,
            time: 0,
        },
    };

    IMP_ISP_Tuning_GetExpr(&mut vexpr);
    match vexpr {
        IMPISPExpr { g_attr } => {
            println!("g_attr mode {}", vexpr.g_attr.mode);
            println!("g_attr time {}", vexpr.g_attr.integration_time);
            println!("g_attr tmin {}", vexpr.g_attr.integration_time_min);
            println!("g_attr tmax {}", vexpr.g_attr.integration_time_max);
            println!("g_attr inus {}", vexpr.g_attr.one_line_expr_in_us);
        }
        IMPISPExpr { s_attr } => {
            println!("s_attr mode {}", vexpr.s_attr.mode);
            println!("s_attr unit {}", vexpr.s_attr.unit);
            println!("s_attr time {}", vexpr.s_attr.time);
        }
    };

    let mut vwb = IMPISPWB {
        mode: 0,
        rgain: 0,
        bgain: 0,
    };
    IMP_ISP_Tuning_GetWB(&mut vwb);
    println!("wbmode {}", vwb.mode);
    println!("wb rgain {}", vwb.rgain);
    println!("wb bgain {}", vwb.bgain);

    IMP_ISP_Tuning_GetWB_Statis(&mut vwb);
    println!("wbst mode {}", vwb.mode);
    println!("wbst rgain {}", vwb.rgain);
    println!("wbst bgain {}", vwb.bgain);

    IMP_ISP_Tuning_GetWB_GOL_Statis(&mut vwb);
    println!("wbgolst mode {}", vwb.mode);
    println!("wbgolst rgain {}", vwb.rgain);
    println!("wbgolst bgain {}", vwb.bgain);

    let mut vrgb = IMPISPCOEFFTWB {
        rgb_coefft_wb_r: 0,
        rgb_coefft_wb_g: 0,
        rgb_coefft_wb_b: 0,
    };
    IMP_ISP_Tuning_Awb_GetRgbCoefft(&mut vrgb);
    println!("rgb r {}", vrgb.rgb_coefft_wb_r);
    println!("rgb g {}", vrgb.rgb_coefft_wb_g);
    println!("rgb b {}", vrgb.rgb_coefft_wb_b);

    let mut vu = 0;
    IMP_ISP_Tuning_GetMaxAgain(&mut vu);
    println!("max again {}", vu);
    IMP_ISP_Tuning_GetMaxDgain(&mut vu);
    println!("max dgain {}", vu);
    IMP_ISP_Tuning_GetHiLightDepress(&mut vu);
    println!("hi light depress {}", vu);

    let mut eva_attr = IMPISPEVAttr {
        ev: 0,
        expr_us: 0,
        ev_log2: 0,
        again: 0,
        dgain: 0,
        gain_log2: 0,
    };

    IMP_ISP_Tuning_GetEVAttr(&mut eva_attr);
    dbg!(eva_attr);

    let mut gamma = IMPISPGamma { gamma: [0u16; 129] };
    IMP_ISP_Tuning_GetGamma(&mut gamma);
    dbg!(gamma);

    //let mut gamma2 = IMPISPGamma { gamma: [0u16; 129] };
    //IMP_ISP_Tuning_SetGamma(&mut gamma2);

    //let mut gamma = IMPISPGamma { gamma: [0u16; 129] };
    //IMP_ISP_Tuning_GetGamma(&mut gamma);

    let mut comp = 0;
    IMP_ISP_Tuning_GetAeComp(&mut comp);
    IMP_ISP_Tuning_SetAeComp(10 * 0x19 + 0x80);
    dbg!(comp);
    IMP_ISP_Tuning_GetAeLuma(&mut comp);
    dbg!(comp);

    let mut m = isp_core_rgb_coefft_wb_attr {
        rgb_coefft_wb_r: 0,
        rgb_coefft_wb_g: 0,
        rgb_coefft_wb_b: 0,
    };

    IMP_ISP_Tuning_Awb_GetRgbCoefft(&mut m);
}

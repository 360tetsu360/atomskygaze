#![allow(clippy::missing_safety_doc)]

use crate::constellation::*;
use fitsio::errors::Result;
use fitsio::hdu::FitsHdu;
use fitsio::FitsFile;
use std::alloc::{alloc, Layout};
use std::ffi::CString;
use std::slice::from_raw_parts;
use wrapper::*;

#[rustfmt::skip]
mod constellation;
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(unused_imports)]
#[allow(improper_ctypes)]
#[allow(clippy::approx_constant)]
#[allow(clippy::upper_case_acronyms)]
#[allow(clippy::type_complexity)]
mod wrapper;

#[rustfmt::skip]
#[allow(clippy::excessive_precision)]
const CONV_FILTER: [f32; 81] = [
    8.504_937_5e-2, 1.458_161_3e-1, 2.143_109_9e-1, 2.700_149_3e-1, 2.916_322_6e-1, 2.700_149_3e-1, 2.143_109_9e-1, 1.458_161_3e-1, 8.504_937_5e-2,
    1.458_161_3e-1, 2.500_000_0e-1, 3.674_336_2e-1, 4.629_373_5e-1, 5.000_000_0e-1, 4.629_373_5e-1, 3.674_336_2e-1, 2.500_000_0e-1, 1.458_161_3e-1,
    2.143_109_9e-1, 3.674_336_2e-1, 5.400_298_6e-1, 6.803_950_0e-1, 7.348_672_4e-1, 6.803_950_0e-1, 5.400_298_6e-1, 3.674_336_2e-1, 2.143_109_9e-1,
    2.700_149_3e-1, 4.629_373_5e-1, 6.803_950_0e-1, 8.572_439_8e-1, 9.258_747_1e-1, 8.572_439_8e-1, 6.803_950_0e-1, 4.629_373_5e-1, 2.700_149_3e-1,
    2.916_322_6e-1, 5.000_000_0e-1, 7.348_672_4e-1, 9.258_747_1e-1, 1.000_000_0e-0, 9.258_747_1e-1, 7.348_672_4e-1, 5.000_000_0e-1, 2.916_322_6e-1,
    2.700_149_3e-1, 4.629_373_5e-1, 6.803_950_0e-1, 8.572_439_8e-1, 9.258_747_1e-1, 8.572_439_8e-1, 6.803_950_0e-1, 4.629_373_5e-1, 2.700_149_3e-1,
    2.143_109_9e-1, 3.674_336_2e-1, 5.400_298_6e-1, 6.803_950_0e-1, 7.348_672_4e-1, 6.803_950_0e-1, 5.400_298_6e-1, 3.674_336_2e-1, 2.143_109_9e-1,
    1.458_161_3e-1, 2.500_000_0e-1, 3.674_336_2e-1, 4.629_373_5e-1, 5.000_000_0e-1, 4.629_373_5e-1, 3.674_336_2e-1, 2.500_000_0e-1, 1.458_161_3e-1,
    8.504_937_5e-2, 1.458_161_3e-1, 2.143_109_9e-1, 2.700_149_3e-1, 2.916_322_6e-1, 2.700_149_3e-1, 2.143_109_9e-1, 1.458_161_3e-1, 8.504_937_5e-2
];
const MAX_ELLIPSE: f32 = 1.5;
const KEEP_NUM: usize = 50;
const THRESH: f32 = 1.0;
const LOG_RATIO: f64 = 20.72326583694641;
const DEFAULT_BAIL_LOG: f64 = -230.25850929940458;
// ATOM Cam2
const SCALE_LOW: f64 = 0.14;
const SCALE_HIGH: f64 = 0.17;
const MIN_WIDTH: f64 = 90.;
const MAX_WIDTH: f64 = 110.;

const INDEX_DIR: &str = "/media/mmc/assets/solver/index";

#[derive(Clone, Debug)]
pub struct Catalog {
    pub width: usize,
    pub height: usize,
    pub rms: f32,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub flux: Vec<f64>,
}

pub fn extract(
    image: &mut [u8],
    mask: Option<&[u8]>,
    width: usize,
    height: usize,
) -> Option<Catalog> {
    let (mask_ptr, mdtype) = match mask {
        Some(buf) => (buf.as_ptr() as *const ::std::os::raw::c_void, SEP_TBYTE),
        None => (std::ptr::null(), 0),
    };
    let image_ = sep_image {
        data: image.as_ptr() as *const ::std::os::raw::c_void,
        noise: std::ptr::null(),
        mask: mask_ptr,
        segmap: std::ptr::null(),
        dtype: SEP_TBYTE as i32,
        ndtype: 0,
        mdtype: mdtype as i32,
        sdtype: 0,
        w: width as i32,
        h: height as i32,
        noiseval: 0.,
        noise_type: SEP_NOISE_NONE as i16,
        gain: 1.,
        maskthresh: 0.,
    };

    let mut bkg: *mut sep_bkg = std::ptr::null_mut();
    let mut catalog: *mut sep_catalog = std::ptr::null_mut();

    unsafe {
        if sep_background(&image_, 64, 64, 3, 3, 0., &mut bkg) != 0 {
            sep_bkg_free(bkg);
            return None;
        }

        if sep_bkg_subarray(
            bkg,
            image.as_mut_ptr() as *mut ::std::os::raw::c_void,
            SEP_TBYTE as i32,
        ) != 0
        {
            sep_bkg_free(bkg);
            return None;
        }

        let thresh = THRESH * (*bkg).globalrms;

        if sep_extract(
            &image_,
            thresh,
            SEP_THRESH_ABS as i32,
            10,
            CONV_FILTER.as_ptr(),
            9,
            9,
            SEP_FILTER_CONV as i32,
            32,
            0.005,
            1,
            1.,
            &mut catalog,
        ) != 0
        {
            sep_bkg_free(bkg);
            sep_catalog_free(catalog);
            return None;
        }

        let nobj = (*catalog).nobj as usize;

        let x_list = from_raw_parts((*catalog).x as *const f64, nobj);
        let y_list = from_raw_parts((*catalog).y as *const f64, nobj);
        let flux_list = from_raw_parts((*catalog).flux as *const f32, nobj);
        let a_list = from_raw_parts((*catalog).a as *const f32, nobj);
        let b_list = from_raw_parts((*catalog).b as *const f32, nobj);
        let flag_list = from_raw_parts((*catalog).flag as *const i16, nobj);

        let mut star_list: Vec<((f64, f64), (f64, f64))> = x_list
            .iter()
            .zip(y_list.iter())
            .zip(flux_list.iter())
            .zip(a_list.iter().zip(b_list.iter()).zip(flag_list.iter()))
            .filter(|(_, ((&a, &b), &flag))| {
                (flag & SEP_OBJ_TRUNC as i16) == 0 && b != 0. && (a / b) < MAX_ELLIPSE
            })
            .map(|(((&x, &y), &flux), _)| {
                let mut kron_flag = 0;
                let mut sum = 0.;
                let mut sumerr = 0.;
                let mut kron_area = 0.;

                let x_pos = x + 1.;
                let y_pos = y + 1.;

                sep_sum_circle(
                    &image_,
                    x_pos,
                    y_pos,
                    3.5,
                    0,
                    5,
                    0,
                    &mut sum,
                    &mut sumerr,
                    &mut kron_area,
                    &mut kron_flag,
                );

                let mag = 20. - 2.5 * sum.log10();

                ((x_pos, y_pos), (flux as f64, mag))
            })
            .collect();

        star_list.sort_by(|a, b| a.1 .1.partial_cmp(&b.1 .1).unwrap());

        let (pos_iter, inf_iter): (Vec<_>, Vec<_>) = star_list.into_iter().take(KEEP_NUM).unzip();
        let (sorted_x, sorted_y): (Vec<_>, Vec<_>) = pos_iter.into_iter().unzip();
        let (sorted_flux, _): (Vec<_>, Vec<_>) = inf_iter.into_iter().unzip();

        Some(Catalog {
            width,
            height,
            rms: (*bkg).globalrms,
            x: sorted_x,
            y: sorted_y,
            flux: sorted_flux,
        })
    }
}

fn fits_add_polynomial(
    hdu: &FitsHdu,
    fits: &mut FitsFile,
    name: &str,
    order: i32,
    data: &[[f64; 10]; 10],
) -> Result<()> {
    for i in 0..order {
        for j in 0..order {
            hdu.write_key(
                fits,
                &format!("{}_{}_{}", name, i, j),
                data[i as usize][j as usize],
            )?;
        }
    }
    Ok(())
}

pub struct Solved {
    wcs: sip_t,
}

impl Solved {
    pub unsafe fn save_to_file(&self, path: &str) {
        let c_str = CString::new(path).unwrap();
        sip_write_to_file(&self.wcs, c_str.as_ptr());
    }

    pub unsafe fn save_to_hdu(&self, hdu: &FitsHdu, fits: &mut FitsFile) -> Result<()> {
        // common
        hdu.write_key(fits, "WCSAXES", 2)?;
        if self.wcs.wcstan.sin != 0 {
            hdu.write_key(
                fits,
                "CTYPE1",
                ("RA---SIN-SIP", "SIN projection + SIP distortions"),
            )?;
            hdu.write_key(
                fits,
                "CTYPE2",
                ("DEC--SIN-SIP", "SIN projection + SIP distortions"),
            )?;
        } else {
            hdu.write_key(
                fits,
                "CTYPE1",
                ("RA---TAN-SIP", "TAN (gnomic) projection + SIP distortions"),
            )?;
            hdu.write_key(
                fits,
                "CTYPE2",
                ("DEC--TAN-SIP", "TAN (gnomic) projection + SIP distortions"),
            )?;
        }
        hdu.write_key(
            fits,
            "EQUINOX",
            (2000.0, "Equatorial coordinates definition (yr)"),
        )?;
        hdu.write_key(fits, "LONPOLE", 180.0)?;
        hdu.write_key(fits, "LATPOLE", 0.0)?;
        hdu.write_key(
            fits,
            "CRVAL1",
            (self.wcs.wcstan.crval[0], "RA  of reference point"),
        )?;
        hdu.write_key(
            fits,
            "CRVAL2",
            (self.wcs.wcstan.crval[1], "DEC of reference point"),
        )?;
        hdu.write_key(
            fits,
            "CRPIX1",
            (self.wcs.wcstan.crpix[0], "X reference pixel"),
        )?;
        hdu.write_key(
            fits,
            "CRPIX2",
            (self.wcs.wcstan.crpix[1], "Y reference pixel"),
        )?;
        hdu.write_key(fits, "CUNIT1", ("deg", "X pixel scale units"))?;
        hdu.write_key(fits, "CUNIT2", ("deg", "Y pixel scale units"))?;
        hdu.write_key(
            fits,
            "CD1_1",
            (self.wcs.wcstan.cd[0][0], "Transformation matrix"),
        )?;
        hdu.write_key(fits, "CD1_2", self.wcs.wcstan.cd[0][1])?;
        hdu.write_key(fits, "CD2_1", self.wcs.wcstan.cd[1][0])?;
        hdu.write_key(fits, "CD2_2", self.wcs.wcstan.cd[1][1])?;
        if self.wcs.wcstan.imagew > 0. {
            hdu.write_key(
                fits,
                "IMAGEW",
                (self.wcs.wcstan.imagew, "Image width,  in pixels."),
            )?;
        }
        if self.wcs.wcstan.imageh > 0. {
            hdu.write_key(
                fits,
                "IMAGEH",
                (self.wcs.wcstan.imageh, "Image height, in pixels."),
            )?;
        }
        hdu.write_key(
            fits,
            "A_ORDER",
            (self.wcs.a_order, "Polynomial order, axis 1"),
        )?;
        fits_add_polynomial(hdu, fits, "A", self.wcs.a_order, &self.wcs.a)?;
        hdu.write_key(
            fits,
            "B_ORDER",
            (self.wcs.b_order, "Polynomial order, axis 2"),
        )?;
        fits_add_polynomial(hdu, fits, "B", self.wcs.b_order, &self.wcs.b)?;
        hdu.write_key(
            fits,
            "AP_ORDER",
            (self.wcs.a_order, "Inv polynomial order, axis 1"),
        )?;
        fits_add_polynomial(hdu, fits, "AP", self.wcs.ap_order, &self.wcs.ap)?;
        hdu.write_key(
            fits,
            "A_ORDER",
            (self.wcs.a_order, "Inv polynomial order, axis 2"),
        )?;
        fits_add_polynomial(hdu, fits, "BP", self.wcs.bp_order, &self.wcs.bp)?;
        Ok(())
    }

    pub unsafe fn ra_dec(&self) -> (f64, f64) {
        let mut ra = 0.;
        let mut dec = 0.;
        sip_get_radec_center(&self.wcs, &mut ra, &mut dec);
        (ra, dec)
    }

    pub unsafe fn ra_dec_hms_string(&self) -> (String, String) {
        let mut rabuff = vec![0u8; 32];
        let mut decbuff = vec![0u8; 32];
        sip_get_radec_center_hms_string(
            &self.wcs,
            rabuff.as_mut_ptr() as *mut ::std::os::raw::c_char,
            decbuff.as_mut_ptr() as *mut ::std::os::raw::c_char,
        );
        let rastr = String::from_utf8(rabuff).unwrap();
        let decstr = String::from_utf8(decbuff).unwrap();
        (rastr, decstr)
    }

    pub unsafe fn pixscale(&self) -> f64 {
        sip_pixel_scale(&self.wcs)
    }

    pub unsafe fn orient(&self) -> f64 {
        sip_get_orientation(&self.wcs)
    }

    pub unsafe fn get_constellations(&self) -> Vec<((f64, f64), (f64, f64))> {
        let mut star_line = vec![];
        for c in 0..CONST_NUM {
            let uniqstars = il_new(16);
            let lines = CONSTELLATION_LINES[c];
            for i in 0..CONSTELLATION_NLINES[c] {
                il_insert_unique_ascending(uniqstars, lines[i as usize] as i32);
            }

            let inboundstars = il_new(16);
            let n_unique = il_size(uniqstars);
            let mut n_inbounds = 0;
            for i in 0..n_unique {
                let star = il_get(uniqstars, i);
                let ra = STAR_POSITIONS[(star * 2) as usize];
                let dec = STAR_POSITIONS[(star * 2 + 1) as usize];

                let mut px = 0.;
                let mut py = 0.;
                if sip_radec2pixelxy(&self.wcs, ra, dec, &mut px, &mut py) == 0 {
                    continue;
                }

                if px < 0. || py < 0. || px > 640. || py > 360. {
                    continue;
                }

                n_inbounds += 1;
                il_append(inboundstars, star);
            }
            il_free(uniqstars);

            if n_inbounds < 2 {
                il_free(inboundstars);
                continue;
            }

            let lines = il_new(16);
            for i in 0..2 * CONSTELLATION_NLINES[c] {
                il_append(lines, CONSTELLATION_LINES[c][i as usize] as i32);
            }

            for i in 0..il_size(lines) / 2 {
                let star1 = il_get(lines, i * 2);
                let star2 = il_get(lines, i * 2 + 1);
                let ra1 = STAR_POSITIONS[(star1 * 2) as usize];
                let dec1 = STAR_POSITIONS[(star1 * 2 + 1) as usize];
                let ra2 = STAR_POSITIONS[(star2 * 2) as usize];
                let dec2 = STAR_POSITIONS[(star2 * 2 + 1) as usize];
                let mut px1 = 0.;
                let mut py1 = 0.;
                let mut px2 = 0.;
                let mut py2 = 0.;
                if sip_radec2pixelxy(&self.wcs, ra1, dec1, &mut px1, &mut py1) == 0
                    || sip_radec2pixelxy(&self.wcs, ra2, dec2, &mut px2, &mut py2) == 0
                {
                    continue;
                }

                star_line.push(((px1, py1), (px2, py2)));
            }

            il_free(inboundstars);
            il_free(lines);
        }

        star_line
    }
}

pub fn solve_field(mut catalog: Catalog, use_flux: bool) -> Option<Solved> {
    unsafe {
        let starxy_layout = Layout::new::<starxy_t>();
        let field_to_solve = alloc(starxy_layout) as *mut starxy_t;
        (*field_to_solve).N = catalog.x.len() as i32;
        (*field_to_solve).x = catalog.x.as_mut_ptr();
        (*field_to_solve).y = catalog.y.as_mut_ptr();
        if use_flux {
            (*field_to_solve).flux = catalog.flux.as_mut_ptr();
        } else {
            (*field_to_solve).flux = std::ptr::null_mut();
        }
        (*field_to_solve).background = std::ptr::null_mut();

        let job_layout = Layout::new::<job_t>();
        let job = alloc(job_layout) as *mut job_t;
        (*job).scales = dl_new(8);
        (*job).depths = il_new(8);
        let bp = &mut (*job).bp;
        blind_init(bp);
        solver_set_default_values(&mut bp.solver);

        bp.solver.fieldxy = field_to_solve;
        bp.solver.field_maxx = catalog.width as f64;
        bp.solver.field_maxy = catalog.height as f64;
        bp.solver.set_crpix = 1;
        bp.solver.set_crpix_center = 1;
        bp.logratio_tosolve = LOG_RATIO;
        bp.solver.logratio_tokeep = LOG_RATIO;
        bp.solver.logratio_totune = LOG_RATIO;
        bp.solver.logratio_bail_threshold = DEFAULT_BAIL_LOG;
        bp.best_hit_only = 1;
        bp.solver.parity = 2;
        bp.solver.do_tweak = 1;
        bp.solver.tweak_aborder = 2;
        bp.solver.tweak_abporder = 2;

        let appl = deg2arcsec(SCALE_LOW);
        let appu = deg2arcsec(SCALE_HIGH);
        bp.solver.funits_lower = appl;
        bp.solver.funits_upper = appu;
        blind_add_field_range(bp, appl.round() as i32, appu.round() as i32);
        dl_append((*job).scales, appl);
        dl_append((*job).scales, appu);

        blind_add_field(&mut (*job).bp, 1);

        let engine = engine_new();
        (*engine).inparallel = 1;
        (*engine).minwidth = MIN_WIDTH;
        (*engine).maxwidth = MAX_WIDTH;

        let c_str = CString::new(INDEX_DIR).unwrap();
        engine_add_search_path(engine, c_str.as_ptr() as *const ::std::os::raw::c_char);
        engine_autoindex_search_paths(engine);

        if il_size((*engine).default_depths) == 0 {
            for i in 1..21 {
                il_append((*engine).default_depths, i * 10);
            }
        }
        if il_size((*job).depths) == 0 {
            if (*engine).inparallel != 0 {
                il_append((*job).depths, 0);
                il_append((*job).depths, 0);
            } else {
                il_append_list((*job).depths, (*engine).default_depths);
            }
        }

        (*job).bp.timelimit = 600;
        (*job).bp.total_timelimit = 600.;

        engine_run_job(engine, job);

        let res = if !(*job).bp.solver.best_match.sip.is_null() {
            let wcs = *(*job).bp.solver.best_match.sip;
            Some(Solved { wcs })
        } else {
            None
        };

        engine_free(engine);
        bl_free((*job).scales);
        dl_free((*job).depths);
        for i in 0..bl_size((*job).bp.solutions) {
            let mo = bl_access((*job).bp.solutions, i) as *mut MatchObj;
            blind_free_matchobj(mo);
        }
        solver_cleanup(&mut (*job).bp.solver);
        blind_cleanup(&mut (*job).bp);

        res
    }
}

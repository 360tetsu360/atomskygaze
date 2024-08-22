const C_SOURCE: [&str; 64] = [
    // qfits
    "ext/astrometry/qfits-an/anqfits.c",
    "ext/astrometry/qfits-an/qfits_card.c",
    "ext/astrometry/qfits-an/qfits_convert.c",
    "ext/astrometry/qfits-an/qfits_error.c",
    "ext/astrometry/qfits-an/qfits_header.c",
    "ext/astrometry/qfits-an/qfits_image.c",
    "ext/astrometry/qfits-an/qfits_table.c",
    "ext/astrometry/qfits-an/qfits_time.c",
    "ext/astrometry/qfits-an/qfits_tools.c",
    "ext/astrometry/qfits-an/qfits_byteswap.c",
    "ext/astrometry/qfits-an/qfits_memory.c",
    "ext/astrometry/qfits-an/qfits_rw.c",
    "ext/astrometry/qfits-an/qfits_float.c",
    // libkd
    "ext/astrometry/libkd/kdint_ddd.c",
    "ext/astrometry/libkd/kdint_fff.c",
    "ext/astrometry/libkd/kdint_ddu.c",
    "ext/astrometry/libkd/kdint_duu.c",
    "ext/astrometry/libkd/kdint_dds.c",
    "ext/astrometry/libkd/kdint_dss.c",
    "ext/astrometry/libkd/kdtree.c",
    "ext/astrometry/libkd/kdtree_dim.c",
    "ext/astrometry/libkd/kdtree_mem.c",
    "ext/astrometry/libkd/kdtree_fits_io.c",
    // anbase
    "ext/astrometry/util/starutil.c",
    "ext/astrometry/util/mathutil.c",
    "ext/astrometry/util/bl-sort.c",
    "ext/astrometry/util/bl.c",
    "ext/astrometry/util/healpix.c",
    "ext/astrometry/util/permutedsort.c",
    "ext/astrometry/util/ioutils.c",
    "ext/astrometry/util/os-features.c",
    "ext/astrometry/util/errors.c",
    "ext/astrometry/util/log.c",
    "ext/astrometry/util/datalog.c",
    // anutil
    "ext/astrometry/util/sip-utils.c",
    "ext/astrometry/util/sip_qfits.c",
    "ext/astrometry/util/fit-wcs.c",
    "ext/astrometry/util/sip.c",
    "ext/astrometry/util/gslutils.c",
    "ext/astrometry/util/fitsioutils.c",
    "ext/astrometry/util/fitstable.c",
    "ext/astrometry/util/fitsbin.c",
    "ext/astrometry/util/fitsfile.c",
    "ext/astrometry/util/tic.c",
    // anfiles
    "ext/astrometry/util/index.c",
    "ext/astrometry/util/codekd.c",
    "ext/astrometry/util/starkd.c",
    "ext/astrometry/util/starxy.c",
    "ext/astrometry/util/quadfile.c",
    // blind
    "ext/astrometry/blind/engine.c",
    "ext/astrometry/blind/blind.c",
    "ext/astrometry/blind/solver.c",
    "ext/astrometry/blind/quad-utils.c",
    "ext/astrometry/blind/matchobj.c",
    "ext/astrometry/blind/tweak2.c",
    "ext/astrometry/blind/verify.c",
    // sep
    "ext/sep/analyse.c",
    "ext/sep/aperture.c",
    "ext/sep/background.c",
    "ext/sep/convolve.c",
    "ext/sep/deblend.c",
    "ext/sep/extract.c",
    "ext/sep/lutz.c",
    "ext/sep/util.c",
];

fn main() {
    println!("cargo::rerun-if-changed=ext");
    cc::Build::new()
        .warnings(false)
        .extra_warnings(false)
        .flag_if_supported("-Wno-unused-result")
        .flag_if_supported("-Wno-implicit-function-declaration")
        .flag_if_supported("-Wno-format-overflow")
        .files(&C_SOURCE)
        // includes
        .include("ext/astrometry")
        .include("ext/astrometry/include")
        .include("ext/astrometry/include/astrometry")
        .compile("stellarsolver");

    println!("cargo:rustc-link-lib=gsl");
    println!("cargo:rustc-link-lib=gslcblas");
}

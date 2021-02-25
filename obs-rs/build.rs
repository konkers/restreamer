extern crate bindgen;

use std::path::PathBuf;
use std::{collections::HashSet, env};

use bindgen::Builder;

// Macro filtering from:
// https://github.com/rust-lang/rust-bindgen/issues/687#issuecomment-450750547
#[derive(Debug)]
struct IgnoreMacros(HashSet<&'static str>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

fn ignore_u128(builder: Builder) -> Builder {
    let ignored_funcs = vec![
        "wcstold",
        "strtold",
        "q.cvt",
        "q.cvt_r",
        ".*_vec3",
        ".*_vec4",
        ".*3v",
        ".*4v",
        "gs_matrix_translate",
        "gs_matrix_scale",
        "gs_vertexbuffer_create",
        "gs_clear",
        "gs_vertexbuffer_flush_direct",
        "gs_vertexbuffer_get_data",
        ".*nexttoward.*",
        "__fpclassifyl",
        "__signbitl",
        "__isnanl",
        "__iseqsigl",
        ".*acosl",
        "__isinfl",
        "__finitel",
        "__issignalingl",
        "asinl",
        "__asinl",
        "atanl",
        "__atanl",
        "atan2l",
        "__atan2l",
        "cosl",
        "__cosl",
        "sinl",
        "__sinl",
        "tanl",
        "__tanl",
        "coshl",
        "__coshl",
        "sinhl",
        "__sinhl",
        "tanhl",
        "__tanhl",
        "acoshl",
        "__acoshl",
        "asinhl",
        "__asinhl",
        "atanhl",
        "__atanhl",
        "expl",
        "__expl",
        "frexpl",
        "__frexpl",
        "ldexpl",
        "__ldexpl",
        "logl",
        "__logl",
        "log10l",
        "__log10l",
        "modfl",
        "__modfl",
        "expm1l",
        "__expm1l",
        "log1pl",
        "__log1pl",
        "logbl",
        "__logbl",
        "exp2l",
        "__exp2l",
        "log2l",
        "__log2l",
        "powl",
        "__powl",
        "sqrtl",
        "__sqrtl",
        "hypotl",
        "__hypotl",
        "cbrtl",
        "__cbrtl",
        "ceill",
        "__ceill",
        "fabsl",
        "__fabsl",
        "floorl",
        "__floorl",
        "fmodl",
        "__fmodl",
        "isinfl",
        "finitel",
        "dreml",
        "__dreml",
        "significandl",
        "__significandl",
        "copysignl",
        "__copysignl",
        "nanl",
        "__nanl",
        "isnanl",
        "j0l",
        "__j0l",
        "j1l",
        "__j1l",
        "jnl",
        "__jnl",
        "y0l",
        "__y0l",
        "y1l",
        "__y1l",
        "ynl",
        "__ynl",
        "erfl",
        "__erfl",
        "erfcl",
        "__erfcl",
        "lgammal",
        "__lgammal",
        "tgammal",
        "__tgammal",
        "gammal",
        "__gammal",
        "lgammal_r",
        "__lgammal_r",
        "rintl",
        "__rintl",
        "nextafterl",
        "__nextafterl",
        "remainderl",
        "__remainderl",
        "scalbnl",
        "__scalbnl",
        "ilogbl",
        "__ilogbl",
        "scalblnl",
        "__scalblnl",
        "nearbyintl",
        "__nearbyintl",
        "roundl",
        "__roundl",
        "truncl",
        "__truncl",
        "remquol",
        "__remquol",
        "lrintl",
        "__lrintl",
        "llrintl",
        "__llrintl",
        "lroundl",
        "__lroundl",
        "llroundl",
        "__llroundl",
        "fdiml",
        "__fdiml",
        "fmaxl",
        "__fmaxl",
        "fminl",
        "__fminl",
        "fmal",
        "__fmal",
        "scalbl",
        "__scalbl",
        "vec4_transform",
        "vec3_plane_dist",
        "vec3_transform",
        "vec3_rotate",
        "vec3_transform3x4",
        "vec3_mirror",
        "vec3_mirrorv",
        "vec3_rand",
        "obs_source_draw_set_color_matrix",
    ];

    ignored_funcs
        .iter()
        .fold(builder, |builder, func| builder.blacklist_function(func))
}
fn main() {
    let ignored_macros = IgnoreMacros(
        vec![
            "FP_NAN",
            "FP_INFINITE",
            "FP_ZERO",
            "FP_SUBNORMAL",
            "FP_NORMAL",
            "FE_INVALID",
            "FE_DIVBYZERO",
            "FE_OVERFLOW",
            "FE_UNDERFLOW",
            "FE_INEXACT",
            "FE_TONEAREST",
            "FE_DOWNWARD",
            "FE_UPWARD",
            "FE_TOWARDZERO",
        ]
        .into_iter()
        .collect(),
    );
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=obs");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let builder = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .parse_callbacks(Box::new(ignored_macros));
    let builder = ignore_u128(builder);
    // Finish the builder and generate the bindings.
    let bindings = builder
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

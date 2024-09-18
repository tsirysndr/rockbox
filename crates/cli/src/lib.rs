use clap::Command;
use owo_colors::OwoColorize;
use std::ffi::CStr;

#[macro_export]
macro_rules! cast_ptr {
    ($ptr:expr) => {{
        #[cfg(target_arch = "aarch64")]
        {
            $ptr as *const u8
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            $ptr as *const i8
        }
    }};
}

#[no_mangle]
pub extern "C" fn parse_args(argc: usize, argv: *const *const u8) -> i32 {
    let string_array = unsafe { std::slice::from_raw_parts(argv, argc) };
    let args: Vec<&str> = string_array
        .iter()
        .map(|&ptr| {
            let c_str = unsafe { CStr::from_ptr(cast_ptr!(ptr)) };
            c_str
                .to_str()
                .unwrap_or("[Invalid UTF-8 or Non Null-Terminated String]")
        })
        .collect();

    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let banner = format!(
        "{}\nA fork of the original Rockbox project, with a focus on modernization and more features.",
        r#"
              __________               __   ___.
    Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
    Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
    Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
    Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
                      \/            \/     \/    \/            \/
    "#
        .yellow()
    );
    let cli = Command::new("rockbox").version(VERSION).about(&banner);

    cli.get_matches_from(args);

    return 0;
}

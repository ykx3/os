#[macro_use]
mod macros;

pub mod logger;
pub use macros::*;

pub const fn get_ascii_header() -> &'static str {
    concat!(
        "\x1B[2J\x1B[1;1H",
        r"
__  __      __  _____            ____  _____
\ \/ /___ _/ /_/ ___/___  ____  / __ \/ ___/
 \  / __ `/ __/\__ \/ _ \/ __ \/ / / /\__ \
 / / /_/ / /_ ___/ /  __/ / / / /_/ /___/ /
/_/\__,_/\__//____/\___/_/ /_/\____//____/

                                       v",
        env!("CARGO_PKG_VERSION"),
        " by 22331116"
    )
}

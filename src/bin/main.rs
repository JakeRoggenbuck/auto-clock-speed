//                  %%%%%%%@%%%%%%
//               %%%%%%%%%%%@%%%%%%%%%%
//            %%%%%%%%%%%%%%@%%%%%%%%%%%
//           %%%%%%%%%             %%%%
//          %%%%%%%         %                           @@@@          @@@@@@@@@      @@@@@@@@@@
//         %%%%%%%         %%%                         @@@@@@       @@@@     @@@@   @@@     @@@@
//         %%%%%%         %%%%%                       @@@  @@@     @@@@             @@@@
//        @@@@@@@       %%%%%%%%%                    @@@    @@@    @@@@               @@@@@@@@
//         %%%%%%       %%%%%%%%%                   @@@@@@@@@@@@   @@@@                     @@@@
//         %%%%%%%         %%%                     @@@@      @@@@   @@@@     @@@@  @@@@     @@@@
//          %%%%%%%                               @@@@        @@@@    @@@@@@@@@      @@@@@@@@@@
//           %%%%%%%%%             %%%%%
//             %%%%%%%%%%%%%@%%%%%%%%%%%%
//               %%%%%%%%%%%@%%%%%%%%%%%
//                    %%%%%%@%%%%%%
//
//
// Automatic CPU frequency scaler and power saver
//
// https://github.com/jakeroggenbuck/auto-clock-speed
// https://autoclockspeed.org
// https://crates.io/crates/autoclockspeed
// https://github.com/autoclockspeed

#![allow(clippy::uninlined_format_args)]

use autoclockspeed::args::parse_args;
use autoclockspeed::config::get_config;

fn main() {
    env_logger::init();

    autoclockspeed::setup::setup();

    let config: autoclockspeed::config::Config = get_config();

    parse_args(config);
}

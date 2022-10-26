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

use args::parse_args;
use config::get_config;
use error::Error;
use log::debug;

pub mod args;
pub mod config;
pub mod cpu;
pub mod daemon;
pub mod display;
pub mod error;
pub mod graph;
pub mod interactive;
pub mod interface;
pub mod logger;
pub mod network;
pub mod power;
pub mod settings;
pub mod setup;
pub mod sysfs;
pub mod system;
pub mod terminal;
pub mod thermal;

fn main() {
    env_logger::init();

    let config: config::Config = get_config();

    parse_args(config);

    setup::setup();
}

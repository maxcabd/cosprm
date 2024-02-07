mod cfg;
mod param;

use std::{thread, time};
use cfg::CostumeAddConfig;
use clap::Parser;
use nuccbin::NuccBinaryType;
use param::{add_entry::*, nucc_binary_handler::*};
use std::path::Path;

#[derive(Parser, Debug)]
#[clap(name = "cosprm", version = "0.1.0", author = "dei", about = "A tool to add costume entries to NSC param files.")]
struct Args {
    #[clap(short, long)]
    json: String,
    #[clap(short, long)]
    dir: String,
}

fn main() {
    let args = Args::parse();

    let cfg = CostumeAddConfig::read_cfg(args.json.as_str());

    let directory = Path::new(args.dir.as_str());

    let mut nucc_binaries = get_nucc_binaries(&directory);

    // Check if each required NUCC binary type is present in the directory
    let required_nucc_types = vec![
        NuccBinaryType::MessageInfo,
        NuccBinaryType::PlayerSettingParam,
        NuccBinaryType::CostumeParam,
        NuccBinaryType::PlayerIcon,
        NuccBinaryType::CharacterSelectParam,
        NuccBinaryType::CostumeBreakParam,
    ];

    for costume in &cfg.costumes {
        for nucc_type in &required_nucc_types {
            if !nucc_binaries.contains_key(nucc_type) {
                // Handle the case when the NUCC binary type is missing
                println!(
                    "NUCC binary type {:?} is missing from the directory.",
                    nucc_type
                );
            } else {
                match nucc_type {
                    NuccBinaryType::MessageInfo => {
                        add_message_info_entry(&mut nucc_binaries, &cfg);
                    }

                    NuccBinaryType::PlayerSettingParam => {
                        add_player_setting_entry(&mut nucc_binaries, &cfg);
                    }

                    NuccBinaryType::CostumeParam => {
                        add_costume_entry(&mut nucc_binaries, &cfg);
                    }

                    NuccBinaryType::PlayerIcon => {
                        add_icon_entry(&mut nucc_binaries, &cfg);
                    }

                    NuccBinaryType::CharacterSelectParam => {
                        add_character_select_entry(&mut nucc_binaries, &cfg);
                    }

                    NuccBinaryType::CostumeBreakParam => {
                        add_costume_break_entry(&mut nucc_binaries, &cfg);
                    }

                    _ => {}
                }
            }
        }

        println!(
            "Adding costume {}bod1 for {}",
            costume.modelcode, costume.characode
        );
    }

    save_nucc_binaries(&directory, &mut nucc_binaries);

    println!("Costume entries added successfully...");
    println!("Exiting...");
    thread::sleep(time::Duration::from_secs(2));

}

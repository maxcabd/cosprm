use crate::cfg::CostumeAddConfig;
use nuccbin::{
    nucc_binary::{
        CharacterSelectParam, CostumeBreakParam, CostumeParam, MessageInfo, NuccBinaryParsed,
        PlayerIcon, PlayerSettingParam,
    },
    NuccBinaryType,
};

use std::collections::HashMap;
use super::calc_crc32;

pub fn add_message_info_entry(
    nucc_binaries: &mut HashMap<NuccBinaryType, Box<dyn NuccBinaryParsed>>,
    cfg: &CostumeAddConfig,
) {
    let message_info = nucc_binaries
        .get_mut(&NuccBinaryType::MessageInfo)
        .unwrap()
        .downcast_mut::<MessageInfo>()
        .unwrap();

    let mut entries = Vec::new();

    for costume in cfg.costumes.iter() {
        let costume_name_exists = message_info.entries.iter().any(|entry| {
            entry.text3 == costume.costume_name && entry.crc32 == calc_crc32(&costume.costume_id)
        });
        let char_name_exists = message_info.entries.iter().any(|entry| {
            entry.text3 == costume.char_name && entry.crc32 == calc_crc32(&costume.cha_id)
        });

        let name_entry = message_info
            .entries
            .iter_mut()
            .find(|entry| entry.crc32 == [246, 160, 24, 181]) // Some random crc32 value that exists for a costume name
            .unwrap();

        let mut chara_name_entry = name_entry;
        chara_name_entry.crc32 = calc_crc32(&costume.cha_id);
        chara_name_entry.text3 = costume.char_name.clone();


        let mut costume_name_entry = chara_name_entry.clone();
        costume_name_entry.crc32 = calc_crc32(&costume.costume_id);
        costume_name_entry.text3 = costume.costume_name.clone();

        if char_name_exists {
            continue;
        }
        entries.push(chara_name_entry.clone());
     
        if costume_name_exists {
            continue;
        }
        entries.push(costume_name_entry.clone());

    }

    message_info.entries.extend(entries);
}

pub fn add_player_setting_entry(
    nucc_binaries: &mut HashMap<NuccBinaryType, Box<dyn NuccBinaryParsed>>,
    cfg: &CostumeAddConfig,
) {
    let player_setting = nucc_binaries
        .get_mut(&NuccBinaryType::PlayerSettingParam)
        .and_then(|param| param.downcast_mut::<PlayerSettingParam>())
        .expect("Failed to retrieve PlayerSettingParam");

    let mut highest_id = player_setting
        .entries
        .iter()
        .map(|entry| entry.player_setting_id)
        .max()
        .unwrap_or_default();

        let mut sorted_costumes = cfg.costumes.clone();
        sorted_costumes.sort_by(|a, b| {
            a.characode
                .cmp(&b.characode)
                .then(a.model_index.cmp(&b.model_index))
        });

    for costume in sorted_costumes.iter() {
         let not_exist = player_setting.entries.iter().any(|entry| {
            entry.cha_b_id == costume.cha_id
                && entry.duel_player_param_model_index == costume.model_index
        });


        let main_entry = player_setting
            .entries
            .iter_mut()
            .filter(|entry| entry.searchcode.contains(&costume.characode))
            .max_by_key(|entry| entry.player_setting_id)
            .unwrap();

        
        let mut entry = main_entry.clone();

        entry.player_setting_id = highest_id + 1;
        entry.duel_player_param_model_index = costume.model_index;

        entry.searchcode = format!(
            "{}{:02}",
            &entry.searchcode.chars().take(4).collect::<String>(),
            entry.searchcode.chars().nth(5).unwrap().to_digit(10).unwrap() + 1
        );

        entry.cha_b_id = costume.cha_id.clone();

        highest_id += 1; // Increment the highest id for the next entry

        // Only push if the entry doesn't already exist
        if not_exist {
            continue;
        }

        player_setting.entries.push(entry);
    }
}

pub fn add_costume_entry(
    nucc_binaries: &mut HashMap<NuccBinaryType, Box<dyn NuccBinaryParsed>>,
    cfg: &CostumeAddConfig,
) {
    let player_setting = {
        let player_setting_ref = nucc_binaries
            .get(&NuccBinaryType::PlayerSettingParam)
            .unwrap()
            .downcast_ref::<PlayerSettingParam>()
            .unwrap();
        player_setting_ref.clone() // Clone the reference
    };

    let costume_param = nucc_binaries
        .get_mut(&NuccBinaryType::CostumeParam)
        .unwrap()
        .downcast_mut::<CostumeParam>()
        .unwrap();

    let mut entries_clone = costume_param.entries.clone();

    let mut highest_costume_link = costume_param
        .entries
        .iter()
        .map(|entry| {
            entry
                .costume_link
                .split("_")
                .last()
                .unwrap()
                .parse::<u32>()
                .unwrap()
        })
        .max()
        .unwrap_or(0)
        + 10;

    // Sort the costumes so we add the entries in the correct order
    let mut sorted_costumes = cfg.costumes.clone();
    sorted_costumes.sort_by(|a, b| {
        a.characode
            .cmp(&b.characode)
            .then(b.model_index.cmp(&a.model_index))
    });

    for costume in sorted_costumes.iter() {
        let characode_index = player_setting
            .entries
            .iter()
            .filter(|entry| entry.cha_b_id == costume.cha_id)
            .map(|entry| entry.characode_index)
            .next()
            .unwrap();

        let psp_entry = player_setting
            .entries
            .iter()
            .filter(|entry| entry.characode_index == characode_index)
            .min_by_key(|entry| entry.player_setting_id)
            .unwrap();

        // We only need the base costume entry to get the index to insert the new entries after it
        let main_entry = entries_clone
            .iter()
            .filter(|entry| entry.player_setting_id == psp_entry.player_setting_id)
            .max_by_key(|entry| entry.color_index)
            .unwrap()
            .clone(); // Clone to avoid borrowing

        let main_entry_index = entries_clone
            .iter()
            .position(|entry| entry == &main_entry)
            .unwrap();

        // We need to find our new psp id we added in the player_setting_param
        let highest_psp_id = player_setting
            .entries
            .iter()
            .filter(|entry| entry.cha_b_id == costume.cha_id)
            .map(|entry| entry.player_setting_id)
            .max()
            .unwrap();

        for i in 0..costume.color_count {
            let mut cos_entry = main_entry.clone();
            cos_entry.player_setting_id = highest_psp_id;
            cos_entry.color_index = i as u32;

            // Only push if the psp id doesn't already exist AND the color index doesn't already exist
            let not_exists = costume_param.entries.iter().any(|entry| {
                entry.player_setting_id == cos_entry.player_setting_id
                    && entry.costume_name == costume.cha_id
                    && entry.color_index == cos_entry.color_index
            });

            if not_exists {
                continue;
            }

            cos_entry.price = 0;
            cos_entry.unlock_condition = 1;
            cos_entry.costume_name = costume.cha_id.clone();
            cos_entry.costume_link =
                format!("COSTUME_{:05}", highest_costume_link + (10 * i as u32));

            entries_clone.insert(main_entry_index + 1 + i as usize, cos_entry);
        }

        highest_costume_link += 10 * costume.color_count as u32;
    }

    costume_param.entries = entries_clone;
}

pub fn add_icon_entry(
    nucc_binaries: &mut HashMap<NuccBinaryType, Box<dyn NuccBinaryParsed>>,
    cfg: &CostumeAddConfig,
) {
    let player_setting = {
        let player_setting_ref = nucc_binaries
            .get(&NuccBinaryType::PlayerSettingParam)
            .unwrap()
            .downcast_ref::<PlayerSettingParam>()
            .unwrap();
        player_setting_ref.clone() // Clone the reference
    };

    let player_icon = nucc_binaries
        .get_mut(&NuccBinaryType::PlayerIcon)
        .unwrap()
        .downcast_mut::<PlayerIcon>()
        .unwrap();

    let mut entries = Vec::new();

    for costume in cfg.costumes.iter() {
        let latest_psp_entry = player_setting
            .entries
            .iter()
            .filter(|entry| entry.searchcode.contains(&costume.characode))
            .min_by_key(|entry| entry.player_setting_id)
            .unwrap();

        let characode_index = latest_psp_entry.characode_index;

        let main_entry = player_icon
            .entries
            .iter_mut()
            .filter(|entry| entry.characode_index == characode_index)
            .max_by_key(|entry| entry.duel_player_param_costume_index)
            .unwrap();

        let mut entry = main_entry.clone();

        if costume.iconcode.is_empty() {
            continue;
        }

        entry.icon_id = costume.iconcode.clone();
        entry.duel_player_param_costume_index = costume.model_index;

        let not_exist = player_icon.entries.iter().any(|entry| {
            entry.duel_player_param_costume_index == costume.model_index
                && entry.characode_index == characode_index
        });

        if not_exist {
            continue;
        }


        entries.push(entry.clone());
    }

    player_icon.entries.extend(entries);
}

pub fn add_character_select_entry(
    nucc_binaries: &mut HashMap<NuccBinaryType, Box<dyn NuccBinaryParsed>>,
    cfg: &CostumeAddConfig,
) {
    let player_setting = {
        let player_setting_ref = nucc_binaries
            .get(&NuccBinaryType::PlayerSettingParam)
            .unwrap()
            .downcast_ref::<PlayerSettingParam>()
            .unwrap();
        player_setting_ref.clone() // Clone the reference
    };


    let character_select = nucc_binaries
        .get_mut(&NuccBinaryType::CharacterSelectParam)
        .unwrap()
        .downcast_mut::<CharacterSelectParam>()
        .unwrap();

    let mut entries = Vec::new();

    for costume in cfg.costumes.iter() {
        // This is our playerSetting entry that we just added for the costume
        let latest_psp_entry = player_setting
            .entries
            .iter()
            .filter(|entry| entry.cha_b_id == costume.cha_id && entry.duel_player_param_model_index == costume.model_index)
            .min_by_key(|entry| entry.player_setting_id)
            .unwrap();

        let main_psp_entry = player_setting
            .entries
            .iter()
            .filter(|entry| entry.characode_index == latest_psp_entry.characode_index)
            .min_by_key(|entry| entry.player_setting_id)
            .unwrap();


        let main_base_entry = character_select
            .entries
            .iter()
            .filter(|entry| entry.searchcode == main_psp_entry.searchcode.clone())
            .min_by_key(|entry| entry.costume_slot_index)
            .unwrap();

        
        let latest_base_entry = character_select
            .entries
            .iter()
            .filter(|entry| entry.page_index == main_base_entry.page_index && entry.slot_index == main_base_entry.slot_index)
            .max_by_key(|entry| entry.costume_slot_index)
            .unwrap();

        
        let mut entry = latest_base_entry.clone();

        entry.costume_slot_index = costume.model_index as u32;
        entry.searchcode = latest_psp_entry.searchcode.clone();
        entry.costume_name = costume.costume_id.clone();
        
        let not_exist = character_select.entries.iter().any(|e| {
            e.costume_name == costume.costume_id
                && e.page_index == entry.page_index
                && e.slot_index == entry.slot_index
                && e.searchcode == entry.searchcode
        });

        if not_exist {
            continue;
        }
        
        entries.push(entry);
    }

    character_select.entries.extend(entries);
}

pub fn add_costume_break_entry(
    nucc_binaries: &mut HashMap<NuccBinaryType, Box<dyn NuccBinaryParsed>>,
    cfg: &CostumeAddConfig,
) {
    let player_setting = {
        let player_setting_ref = nucc_binaries
            .get(&NuccBinaryType::PlayerSettingParam)
            .unwrap()
            .downcast_ref::<PlayerSettingParam>()
            .unwrap();
        player_setting_ref.clone() // Clone the reference
    };

    let costume_break = nucc_binaries
        .get_mut(&NuccBinaryType::CostumeBreakParam)
        .unwrap()
        .downcast_mut::<CostumeBreakParam>()
        .unwrap();

    let mut entries = Vec::new();

    for costume in cfg.costumes.iter() {
        if costume.has_costume_break { // Only add costume break entries if set to true
            continue;
        }

        let searchcode = format!("{}00", &costume.characode);

        let psp_entry = player_setting
            .entries
            .iter()
            .filter(|entry| entry.searchcode == searchcode)
            .min_by_key(|entry| entry.player_setting_id)
            .unwrap();

        let characode_index = psp_entry.characode_index;

        let main_entry = costume_break
            .entries
            .iter_mut()
            .filter(|entry| entry.characode_index == characode_index)
            .min_by_key(|entry| entry.costume_index)
            .unwrap();

        let mut entry = main_entry.clone();
        entry.costume_index = costume.model_index as u32;

        let not_exist = costume_break.entries.iter().any(|entry| {
            entry.costume_index == costume.model_index as u32
                && entry.characode_index == characode_index
        });

        if not_exist {
            continue;
        }

        entries.push(entry.clone());
    }

    costume_break.entries.extend(entries);
}

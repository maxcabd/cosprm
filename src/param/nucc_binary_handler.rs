use nuccbin::nucc_binary::{
    NuccBinaryParsed, NuccBinaryParsedDeserializer, NuccBinaryParsedReader, NuccBinaryParsedWriter,
};
use nuccbin::NuccBinaryType;
use std::collections::HashMap;
use std::path::Path;
use strum::IntoEnumIterator;
use walkdir::WalkDir;
use xfbin::{nucc::NuccChunk, read_xfbin, write_xfbin};

const NUCC_BINARY_PATTERNS: [NuccBinaryType; 7] = [
    NuccBinaryType::PrmBas,
    NuccBinaryType::MessageInfo,
    NuccBinaryType::PlayerSettingParam,
    NuccBinaryType::CostumeParam,
    NuccBinaryType::PlayerIcon,
    NuccBinaryType::CharacterSelectParam,
    NuccBinaryType::CostumeBreakParam,
];

//// Gather parsed NUCC binaries from a directory
pub fn get_nucc_binaries(directory: &Path) -> HashMap<NuccBinaryType, Box<dyn NuccBinaryParsed>> {
    let mut nucc_type_parsed = HashMap::new();

    let files = collect_files(&directory);

    files.iter().for_each(|file| {
        let xfbin = read_xfbin(Path::new(file)).unwrap();
        for chunk in xfbin.get_chunks_by_type("nuccChunkBinary") {
            let chunk_info = &xfbin.chunk_table.get_chunk_info(&chunk.chunk_map);

            if let Some(nucc_binary_type) = find_nucc_binary_type(&chunk_info.1) {
                let nucc_binary = chunk.data.as_bytes();

                if NUCC_BINARY_PATTERNS.contains(&nucc_binary_type) {
                    let reader = NuccBinaryParsedReader(nucc_binary_type, &nucc_binary);
                    let nucc_binary_parsed: Box<dyn NuccBinaryParsed> = reader.into();
                    nucc_type_parsed.insert(nucc_binary_type, nucc_binary_parsed);
                }
            }
        }
    });

    if nucc_type_parsed.is_empty() {
        panic!("No valid NUCC binaries found in the directory!");
    }

    nucc_type_parsed
}

pub fn save_nucc_binaries(
    directory: &Path,
    nucc_binaries: &mut HashMap<NuccBinaryType, Box<dyn NuccBinaryParsed>>,
) {
    let files = collect_files(&directory);

    for file in &files {
        let mut xfbin = read_xfbin(Path::new(file)).unwrap();

        let mut updated_chunks = Vec::new();

        for chunk in xfbin.get_chunks_by_type("nuccChunkBinary") {
            let chunk_info = &xfbin.chunk_table.get_chunk_info(&chunk.chunk_map);

            if let Some(nucc_binary_type) = find_nucc_binary_type(&chunk_info.1) {
                if let Some(nucc_binary) = nucc_binaries.get_mut(&nucc_binary_type) {
                    let deserializer =
                        NuccBinaryParsedDeserializer(nucc_binary_type, nucc_binary.serialize());
                    let writer = NuccBinaryParsedWriter(deserializer.into());
                    let bytes: Vec<u8> = writer.into();

                    // Replace the chunk data with the serialized binary chunk
                    let mut updated_chunk = chunk.clone();
                    updated_chunk.size = bytes.len() as u32;
                    updated_chunk.data = NuccChunk::NuccBinary(bytes.clone());

                    // Replace the chunk in the xfbin
                    updated_chunks.push(updated_chunk);
                }
            }
        }

        for u in updated_chunks {
            for page in &mut xfbin.pages {
                for chunk in &mut page.chunks {
                    if chunk.chunk_map == u.chunk_map {
                        *chunk = u.clone();
                    }
                }
            }
        }

        write_xfbin(Path::new(file), &mut xfbin).unwrap();
    }
}

fn find_nucc_binary_type(chunk_filepath: &String) -> Option<NuccBinaryType> {
    for nucc_binary_type in NuccBinaryType::iter() {
        let regex = nucc_binary_type.patterns();
        if regex.is_match(chunk_filepath) {
            return Some(nucc_binary_type);
        }
    }
    None
}

fn collect_files(directory: &Path) -> Vec<String> {
    let mut files = Vec::new();

    for entry in WalkDir::new(directory).follow_links(true) {
        match entry {
            Ok(entry) => {
                // Also only collect .xfbin files
                if entry.file_type().is_file() && entry.path().extension().unwrap() == "xfbin" {
                    files.push(entry.path().to_path_buf());
                }
            }
            Err(e) => eprintln!("Error accessing entry: {}", e),
        }
    }

    files
        .iter()
        .map(|path| path.to_str().unwrap().to_string())
        .collect::<Vec<String>>()
}

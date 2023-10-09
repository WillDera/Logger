use std::fs::Metadata;

use serde::Deserialize;
use validator::Validate;

const MIN_CONSTANT: u64 = 1;
const MAX_CONSTANT: u64 = u64::MAX;

#[derive(Debug, Deserialize, Validate)]
pub struct Pagination {
    #[validate(range(min = "MIN_CONSTANT", max = "MAX_CONSTANT"))]
    pub offset: u32,
    pub limit: u32,
}

pub fn extract_data<T>(array: &Vec<T>, limit: u32, offset: u32) -> &[T]
where
    T: Clone,
{
    // Check if the offset is greater than or equal to the length of the array.
    if offset >= array.len() as u32 {
        return &[];
    }

    // Calculate the upper bound of the slice.
    let upper_bound = std::cmp::min(offset + limit, array.len() as u32);

    // Return a slice of the array from the offset to the upper bound.
    &array[(offset - 1) as usize..upper_bound as usize]
}

pub fn get_latest_file_name(directory_path: &str) -> Result<String, std::io::Error> {
    // Get the metadata of all the files in the directory_path.
    let mut metadata = std::fs::read_dir(directory_path)
        .unwrap()
        .filter_map(|entry| {
            let metadata = entry
                .as_ref()
                .expect("Failed to extract metadata")
                .metadata()
                .unwrap();
            let filename = entry
                .as_ref()
                .expect("Failed to extract Filename")
                .file_name()
                .to_str()
                .unwrap()
                .to_string();

            Some((metadata, filename))
        })
        .collect::<Vec<(Metadata, String)>>();

    // Sort the metadata by the last modified time.
    metadata.sort_by_key(|(metadata, _)| metadata.modified().unwrap());

    // Get the metadata of the most recently modified file
    let (_metadata, filename) = metadata.last().unwrap();

    Ok(filename.to_string())
}

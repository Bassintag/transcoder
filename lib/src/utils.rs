pub fn get_output_file_name(name: &String) -> String {
    let re = regex::Regex::new(r"[^A-Za-z0-9]+").unwrap();
    format!("{}.h264.aac.stereo.remux.mp4", re.replace_all(name, "."),)
}

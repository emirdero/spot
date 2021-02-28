pub struct FileParser {}

impl FileParser {
    /// takes inn a file ending and returns the appropriate content-type header definition
    ///
    /// For example "html" -> "text/html"
    pub fn get_type(file_ending: &str) -> String {
        let supported_file_types = [
            ("html", "text/html"),
            ("css", "text/css"),
            ("json", "application/json"),
            ("js", "application/javascript"),
            ("zip", "application/zip"),
            ("csv", "text/csv"),
            ("xml", "text/xml"),
            ("ico", "image/x-icon"),
            ("jpg", "image/jpeg"),
            ("jpeg", "image/jpeg"),
            ("png", "image/png"),
            ("gif", "image/gif"),
            ("mp3", "audio/mpeg"),
            ("mp4", "video/mp4"),
            ("webm", "video/webm"),
        ];
        for supported_type in supported_file_types.iter() {
            if supported_type.0 == file_ending {
                return String::from(supported_type.1);
            }
        }
        return String::from("text/plain");
    }
}

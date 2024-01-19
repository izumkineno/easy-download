
#[cfg(test)]
mod tests {
    use byte_unit::{Byte, UnitType};
    use easy_download::core::file_request::FileRequest;

    static URL: &str = "https://www.voidtools.com/Everything-1.4.1.1024.x64.Lite-Setup.exe";
    // static URL: &str = "https://github.com/ModOrganizer2/modorganizer/releases/download/v2.5.0/Mod.Organizer-2.5.0.7z";
    // static URL: &str = "https://download.jetbrains.com/rustrover/RustRover-233.13135.116.exe";

    #[tokio::test]
    async fn get_size() {
        let mut request = FileRequest::new(URL);

        let adjusted_byte = request.get_size().await.unwrap();
        let adjusted_byte = Byte::from_u64(adjusted_byte).get_appropriate_unit(UnitType::Binary);

        print!("{adjusted_byte:.2}");
    }

}

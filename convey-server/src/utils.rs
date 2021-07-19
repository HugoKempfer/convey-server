pub fn is_magnet_link_valid(magnet: &str) -> bool {
    &magnet[0..8] != "magnet:?"
}

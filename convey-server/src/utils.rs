pub fn is_magnet_link_valid(magnet: &str) -> bool {
    magnet.len() > 8 && &magnet[0..8] == "magnet:?"
}

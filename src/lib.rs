#[macro_use]
extern crate bitflags;
extern crate open_vcdiff_sys;

use std::os;
use std::ptr;

bitflags! {
    pub flags FormatExtension: u32 {
        const FORMAT_STANDARD    = 0b000,
        const FORMAT_INTERLEAVED = 0b001,
        const FORMAT_CHECKSUM    = 0b010,
        const FORMAT_JSON        = 0b100,
    }
}


pub fn encode(dictionary: &[u8], target: &[u8], format_extensions: FormatExtension, look_for_target_matches: bool) -> Vec<u8> {
    use open_vcdiff_sys::VCDiffFormatExtensionFlagValues::*;

    let mut encoded_data = ptr::null_mut();
    let mut encoded_len = 0;

    let mut flags = VCD_STANDARD_FORMAT as os::raw::c_int;

    if format_extensions.contains(FORMAT_INTERLEAVED) {
        flags = flags | VCD_FORMAT_INTERLEAVED as os::raw::c_int;
    }

    if format_extensions.contains(FORMAT_CHECKSUM) {
        flags = flags | VCD_FORMAT_CHECKSUM as os::raw::c_int;
    }

    if format_extensions.contains(FORMAT_JSON) {
        flags = flags | VCD_FORMAT_JSON as os::raw::c_int;
    }

    unsafe {
        open_vcdiff_sys::encode(dictionary.as_ptr(),
                                dictionary.len(),
                                target.as_ptr(),
                                target.len(),
                                &mut encoded_data,
                                &mut encoded_len,
                                flags,
                                if look_for_target_matches { 1 } else { 0 });
    }

    let mut result = Vec::with_capacity(encoded_len);

    unsafe {
        ptr::copy_nonoverlapping(encoded_data, result.as_mut_ptr(), encoded_len);
        result.set_len(encoded_len);
        open_vcdiff_sys::free_data(encoded_data);
    }

    result
}

pub fn decode(dictionary: &[u8], encoded: &[u8]) -> Vec<u8> {
    let mut target_data = ptr::null_mut();
    let mut target_len = 0;

    unsafe {
        open_vcdiff_sys::decode(dictionary.as_ptr(),
                                dictionary.len(),
                                encoded.as_ptr(),
                                encoded.len(),
                                &mut target_data,
                                &mut target_len);
    }

    let mut result = Vec::with_capacity(target_len);

    unsafe {
        ptr::copy_nonoverlapping(target_data, result.as_mut_ptr(), target_len);
        result.set_len(target_len);
        open_vcdiff_sys::free_data(target_data);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_standard() {
        let dict: &[u8] = &[1, 2, 3];
        let target: &[u8] = &[4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 4];
        let encoded = encode(dict, target, FORMAT_STANDARD, false);
        let decoded = decode(dict, &encoded);

        assert_eq!(target, decoded.as_slice());
    }

    #[test]
    fn roundtrip_standard_target_matches() {
        let dict: &[u8] = &[1, 2, 3];
        let target: &[u8] = &[4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 4];
        let encoded = encode(dict, target, FORMAT_STANDARD, true);
        let decoded = decode(dict, &encoded);

        assert_eq!(target, decoded.as_slice());
    }

    #[test]
    fn roundtrip_interleaved() {
        let dict: &[u8] = &[1, 2, 3];
        let target: &[u8] = &[4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 4];
        let encoded = encode(dict, target, FORMAT_INTERLEAVED, false);
        let decoded = decode(dict, &encoded);

        assert_eq!(target, decoded.as_slice());
    }

    #[test]
    fn roundtrip_interleaved_target_matches() {
        let dict: &[u8] = &[1, 2, 3];
        let target: &[u8] = &[4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 4];
        let encoded = encode(dict, target, FORMAT_INTERLEAVED, true);
        let decoded = decode(dict, &encoded);

        assert_eq!(target, decoded.as_slice());
    }

    #[test]
    fn roundtrip_checksum() {
        let dict: &[u8] = &[1, 2, 3];
        let target: &[u8] = &[4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 4];
        let encoded = encode(dict, target, FORMAT_CHECKSUM, false);
        let decoded = decode(dict, &encoded);

        assert_eq!(target, decoded.as_slice());
    }

    #[test]
    fn roundtrip_checksum_target_matches() {
        let dict: &[u8] = &[1, 2, 3];
        let target: &[u8] = &[4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 4];
        let encoded = encode(dict, target, FORMAT_CHECKSUM, true);
        let decoded = decode(dict, &encoded);

        assert_eq!(target, decoded.as_slice());
    }

    #[test]
    fn roundtrip_interleaved_checksum() {
        let dict: &[u8] = &[1, 2, 3];
        let target: &[u8] = &[4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 4];
        let encoded = encode(dict, target, FORMAT_INTERLEAVED | FORMAT_CHECKSUM, false);
        let decoded = decode(dict, &encoded);

        assert_eq!(target, decoded.as_slice());
    }

    #[test]
    fn roundtrip_interleaved_checksum_target_matches() {
        let dict: &[u8] = &[1, 2, 3];
        let target: &[u8] = &[4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 4];
        let encoded = encode(dict, target, FORMAT_INTERLEAVED | FORMAT_CHECKSUM, true);
        let decoded = decode(dict, &encoded);

        assert_eq!(target, decoded.as_slice());
    }
}

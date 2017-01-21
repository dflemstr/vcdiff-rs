use ::open_vcdiff_sys;

use std::error;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;
use std::slice;
use std::result;

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub struct Error(&'static str);

impl error::Error for Error {
    fn description(&self) -> &str {
        self.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "vcdiff error: {}", self.0)
    }
}

pub type Result<T> = result::Result<T,Error>;

pub trait Output {
    fn append(&mut self, buffer: &[u8]);
    fn clear(&mut self);
    fn reserve(&mut self, bytes: usize);
    fn size(&self) -> usize;
}

impl Output for Vec<u8> {
    fn append(&mut self, buffer: &[u8]) {
        self.extend_from_slice(buffer);
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn reserve(&mut self, bytes: usize) {
        self.reserve(bytes);
    }

    fn size(&self) -> usize {
        self.len()
    }
}

// Set up some Rust functions to call from C which delegate to an Output

unsafe extern "C" fn output_trait_append_cb(callback_pointer: *mut c_void, buffer: *const u8, buffer_size: usize) {
    let mut callback_pointer: Box<&mut Output> = mem::transmute(callback_pointer);
    let buffer: &[u8] = slice::from_raw_parts(buffer, buffer_size);
    callback_pointer.append(buffer);
}

unsafe extern "C" fn output_trait_clear_cb(callback_pointer: *mut c_void) {
    let mut callback_pointer: Box<&mut Output> = mem::transmute(callback_pointer);
    callback_pointer.clear();
}

unsafe extern "C" fn output_trait_reserve_cb(callback_pointer: *mut c_void, bytes: usize) {
    let mut callback_pointer: Box<&mut Output> = mem::transmute(callback_pointer);
    callback_pointer.reserve(bytes);
}

unsafe extern "C" fn output_trait_size_cb(callback_pointer: *const c_void) -> usize {
    let callback_pointer: Box<&Output> = mem::transmute(callback_pointer);
    callback_pointer.size()
}

pub struct Decoder<'d> {
    decoder: *mut c_void,

    // decoder holds a borrow across the FFI boundary; model this as PhantomData
    dictionary: PhantomData<&'d [u8]>,
}

impl<'d> Decoder<'d> {
    pub fn new(dictionary: &'d [u8]) -> Decoder<'d> {
        // make a new decoder
        // this is C++ "operator new" under the hood
        let decoder = unsafe { open_vcdiff_sys::new_decoder() };

        // point it to the dictionary buffer
        // this is a borrow we'll keep for the lifetime of the Decoder, so there's no need to copy it
        unsafe { open_vcdiff_sys::decoder_start_decoding(decoder, dictionary.as_ptr() as *const i8, dictionary.len()) }

        // Decoder is ready to use
        Decoder{
            decoder: decoder,
            dictionary: PhantomData,
        }
    }

    pub fn with_options(
        dictionary: &'d [u8],
        maximum_target_file_size: usize,
        maximum_target_window_size: usize,
        allow_vcd_target: bool
    ) -> Result<Decoder<'d>> {
        // make a new decoder
        // this is C++ "operator new" under the hood
        let decoder = unsafe { open_vcdiff_sys::new_decoder() };

        // wrap it in a decoder object
        let mut decoder = Decoder{
            decoder: decoder,
            dictionary: PhantomData,
        };

        // set options, since we need to do that before start_decoding()
        decoder.set_maximum_target_file_size(maximum_target_file_size)?;
        decoder.set_maximum_target_window_size(maximum_target_window_size)?;
        decoder.set_allow_vcd_target(allow_vcd_target)?;

        // point it to the dictionary buffer
        unsafe { open_vcdiff_sys::decoder_start_decoding(decoder.decoder, dictionary.as_ptr() as *const i8, dictionary.len()) }

        // return the decoder
        Ok(decoder)
    }

    fn set_maximum_target_file_size(&mut self, size: usize) -> Result<()> {
        if unsafe {
            open_vcdiff_sys::decoder_set_maximum_target_file_size(self.decoder, size)
        } {
            Ok(())
        } else {
            Err(Error("unable to set decoder maximum target file size"))
        }
    }

    fn set_maximum_target_window_size(&mut self, size: usize) -> Result<()> {
        if unsafe {
            open_vcdiff_sys::decoder_set_maximum_target_window_size(self.decoder, size)
        } {
            Ok(())
        } else {
            Err(Error("unable to set decoder maximum target window size"))
        }
    }

    fn set_allow_vcd_target(&mut self, allow: bool) -> Result<()> {
        unsafe {
            open_vcdiff_sys::decoder_set_allow_vcd_target(self.decoder, allow);
        }

        // for whatever reason, this call can't fail, but all the others can
        // return a Result anyway for consistency
        Ok(())
    }

    pub fn decode<O: Output>(&mut self, data: &[u8], output: &mut O) -> Result<()> {
        // box the output reference so we can pass it to C and back
        let boxed_output: Box<&mut Output> = Box::new(output);

        // unsafety: decode_chunk() doesn't hold any pointers outside the scope of the call
        // we have a borrow of everything we pass in, so there's no lifetime concerns
        if unsafe {
            open_vcdiff_sys::decoder_decode_chunk_to_callbacks(
                self.decoder,
                data.as_ptr() as *const i8,
                data.len(),
                mem::transmute(boxed_output),
                Some(output_trait_append_cb),
                Some(output_trait_clear_cb),
                Some(output_trait_reserve_cb),
                Some(output_trait_size_cb)
            )
        } {
            Ok(())
        } else {
            Err(Error("decode failed"))
        }
    }

    pub fn finish(self) -> Result<()> {
        if unsafe {
            open_vcdiff_sys::decoder_finish_decoding(self.decoder)
        } {
            Ok(())
        } else {
            Err(Error("finish decoding failed"))
        }
    }
}

impl<'d> Drop for Decoder<'d> {
    fn drop(&mut self)
    {
        // run C++ "operator delete" on the decoder we hold
        unsafe { open_vcdiff_sys::delete_decoder(self.decoder); }
    }
}

#[cfg(test)]
mod tests {

    mod decoder {
        use streaming::*;

        #[test]
        fn create_and_destroy() {
            // creating and destroying a Decoder with an empty dictionary should not panic
            Decoder::new(b"");
        }

        #[test]
        fn create_and_destroy_with_options() {
            // setting some options shouldn't break
            Decoder::with_options(b"", 10 << 20, 10 << 20, false).expect("with_options()");
        }

        #[test]
        fn test_finish_after_decoding_nothing() {
            // it's legal to finish decoding nothing
            let mut decoder = Decoder::new(b"");
            let mut out: Vec<u8> = Vec::new();
            decoder.decode(b"", &mut out).expect("decode");
            decoder.finish().expect("success");
        }

        #[test]
        fn test_finish_without_decoding() {
            // it's not normally legal to finish without starting the decode, but we make it so
            let decoder = Decoder::new(b"");
            decoder.finish().expect("success");
        }

        #[test]
        fn test_decode_garbage() {
            let mut decoder = Decoder::new(b"");
            let mut output: Vec<u8> = Vec::new();

            // attempt to decode something that's not a VCD header
            // this should fail immediately
            match decoder.decode(b"VCD", &mut output) {
                Ok(()) => { panic!("expected decode to fail") }
                Err(_) => {}
            }

            // and the output should still be empty
            assert_eq!(output.len(), 0);
        }

        #[test]
        fn test_finish_with_partial_buffer() {
            let mut decoder = Decoder::new(b"");
            let mut output: Vec<u8> = Vec::new();

            // decode just the start of a valid header
            // this should succeed...
            decoder.decode(b"\xD6\xC3", &mut output).expect("decode");

            // ...but not write anything to the output
            assert_eq!(output.len(), 0);

            // finishing should fail, since there's undecoded bits in the input buffer
            match decoder.finish() {
                Ok(()) => { panic!("expected finish to fail"); }
                Err(_) => {}
            }
        }
    }
}

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
            dictionary: PhantomData
        }
    }

    pub fn set_maximum_target_file_size(&mut self, size: usize) -> Result<()> {
        if unsafe {
            open_vcdiff_sys::decoder_set_maximum_target_file_size(self.decoder, size)
        } {
            Ok(())
        } else {
            Err(Error("unable to set decoder maximum target file size"))
        }
    }

    pub fn set_maximum_target_window_size(&mut self, size: usize) -> Result<()> {
        if unsafe {
            open_vcdiff_sys::decoder_set_maximum_target_window_size(self.decoder, size)
        } {
            Ok(())
        } else {
            Err(Error("unable to set decoder maximum target window size"))
        }
    }

    pub fn set_allow_vcd_target(&mut self, allow: bool) -> Result<()> {
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

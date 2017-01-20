#pragma once

#include <stdbool.h>
#include <stddef.h>  // size_t
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

enum VCDiffFormatExtensionFlagValues {
    // No extensions: the encoded format will conform to the RFC
    // draft standard for VCDIFF.
    VCD_STANDARD_FORMAT = 0x00,
    // If this flag is specified, then the encoder writes each delta file
    // window by interleaving instructions and sizes with their corresponding
    // addresses and data, rather than placing these elements
    // into three separate sections.  This facilitates providing partially
    // decoded results when only a portion of a delta file window is received
    // (e.g. when HTTP over TCP is used as the transmission protocol.)
    VCD_FORMAT_INTERLEAVED = 0x01,
    // If this flag is specified, then an Adler32 checksum
    // of the target window data is included in the delta window.
    VCD_FORMAT_CHECKSUM = 0x02,
    // If this flag is specified, the encoder will output a JSON string
    // instead of the VCDIFF file format. If this flag is set, all other
    // flags have no effect.
    VCD_FORMAT_JSON = 0x04
};

typedef void (*output_callback_append_fn)(void *, const char *, size_t);
typedef void (*output_callback_clear_fn)(void *);
typedef void (*output_callback_reserve_fn)(void *, size_t);
typedef size_t (*output_callback_size_fn)(const void *);

typedef int VCDiffFormatExtensionFlags;

void * new_decoder();
void decoder_start_decoding(void *decoder, const char *dictionary_ptr, size_t dictionary_size);
bool decoder_set_maximum_target_file_size(void *decoder, size_t new_maximum_target_file_size);
bool decoder_set_maximum_target_window_size(void *decoder, size_t new_maximum_target_window_size);
void decoder_set_allow_vcd_target(void *decoder, bool allow_vcd_target);
bool decoder_decode_chunk_to_callbacks(
    void *decoder, const char *data, size_t len,
    void * pointer,
    output_callback_append_fn append_fn,
    output_callback_clear_fn clear_fn,
    output_callback_reserve_fn reserve_fn,
    output_callback_size_fn size_fn
    );
bool decoder_finish_decoding(void *decoder);
void delete_decoder(void *decoder);


void vcdiff_encode(const uint8_t *dictionary_data,
                   size_t dictionary_len,
                   const uint8_t *target_data,
                   size_t target_len,
                   uint8_t **encoded_data,
                   size_t *encoded_len,
                   VCDiffFormatExtensionFlags flags,
                   bool look_for_target_matches);

void vcdiff_free_data(uint8_t *data);

#ifdef __cplusplus
} /* extern "C" */
#endif

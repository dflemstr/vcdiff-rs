#include "glue.h"
#include <stdlib.h>
#include <string.h>
#include "google/vcencoder.h"
#include "google/vcdecoder.h"

extern "C" {

void vcdiff_encode(const uint8_t *dictionary_data,
                   size_t dictionary_len,
                   const uint8_t *target_data,
                   size_t target_len,
                   uint8_t **encoded_data,
                   size_t *encoded_len,
                   VCDiffFormatExtensionFlags flags,
                   bool look_for_target_matches) {
    std::string encoded;

    open_vcdiff::VCDiffEncoder encoder((const char *)dictionary_data, dictionary_len);

    encoder.SetFormatFlags(flags);
    encoder.SetTargetMatching(look_for_target_matches);

    encoder.Encode((const char *) target_data, target_len, &encoded);

    *encoded_data = (uint8_t *) malloc(encoded.size());
    memcpy(*encoded_data, encoded.data(), encoded.size());
    *encoded_len = encoded.size();
}

void vcdiff_decode(const uint8_t *dictionary_data,
                   size_t dictionary_len,
                   const uint8_t *encoded_data,
                   size_t encoded_len,
                   uint8_t **target_data,
                   size_t *target_len) {
    open_vcdiff::VCDiffDecoder decoder;
    std::string encoded((const char *) encoded_data, encoded_len);
    std::string target;

    decoder.Decode((const char *) dictionary_data,
                   dictionary_len,
                   encoded,
                   &target);

    *target_data = (uint8_t *) malloc(target.size());
    memcpy(*target_data, target.data(), target.size());
    *target_len = target.size();
}

void vcdiff_free_data(uint8_t *data) {
    free(data);
}

}

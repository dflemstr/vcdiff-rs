#include "glue.h"
#include <stdlib.h>
#include <string.h>
#include "google/vcencoder.h"
#include "google/vcdecoder.h"
#include "google/output_string.h"

class OutputCallbackInterface : public open_vcdiff::OutputStringInterface {
public:
    OutputCallbackInterface(
        void * pointer,
        output_callback_append_fn append,
        output_callback_clear_fn clear,
        output_callback_reserve_fn reserve,
        output_callback_size_fn size
        );
    virtual ~OutputCallbackInterface();
    virtual OutputCallbackInterface& append(const char* s, size_t n);
    virtual void clear();
    virtual void push_back(char c);
    virtual void ReserveAdditionalBytes(size_t res_arg);
    virtual size_t size() const;
private:
    void * pointer;
    output_callback_append_fn append_fn;
    output_callback_clear_fn clear_fn;
    output_callback_reserve_fn reserve_fn;
    output_callback_size_fn size_fn;
};

OutputCallbackInterface::OutputCallbackInterface(
    void * pointer,
    output_callback_append_fn append,
    output_callback_clear_fn clear,
    output_callback_reserve_fn reserve,
    output_callback_size_fn size
) : pointer(pointer), append_fn(append), clear_fn(clear), reserve_fn(reserve), size_fn(size) {
}

OutputCallbackInterface::~OutputCallbackInterface() {
}

OutputCallbackInterface& OutputCallbackInterface::append(const char *s, size_t n) {
    append_fn(pointer, s, n);
    return *this;       // like std::string
}

void OutputCallbackInterface::push_back(char c) {
    // implement in terms of append
    append_fn(pointer, &c, 1);
}

void OutputCallbackInterface::clear() {
    clear_fn(pointer);
}

void OutputCallbackInterface::ReserveAdditionalBytes(size_t n) {
    reserve_fn(pointer, n);
}

size_t OutputCallbackInterface::size() const {
    return size_fn(pointer);
}


struct callback_decoder {
    callback_decoder(OutputCallbackInterface callback_interface);

    open_vcdiff::VCDiffStreamingDecoder decoder;
    OutputCallbackInterface callback_interface;
};

callback_decoder::callback_decoder(OutputCallbackInterface callback_interface)
    : callback_interface(callback_interface)
{
}

extern "C" {

void * new_decoder() {
    return (void *)(new open_vcdiff::VCDiffStreamingDecoder());
}

void decoder_start_decoding(void *decoder, const char *dictionary_ptr, size_t dictionary_size) {
    ((open_vcdiff::VCDiffStreamingDecoder *)decoder)->StartDecoding(dictionary_ptr, dictionary_size);
}

bool decoder_set_maximum_target_file_size(void *decoder, size_t new_maximum_target_file_size) {
    return ((open_vcdiff::VCDiffStreamingDecoder *)decoder)->SetMaximumTargetFileSize(new_maximum_target_file_size);
}

bool decoder_set_maximum_target_window_size(void *decoder, size_t new_maximum_target_window_size) {
    return ((open_vcdiff::VCDiffStreamingDecoder *)decoder)->SetMaximumTargetWindowSize(new_maximum_target_window_size);
}

void decoder_set_allow_vcd_target(void *decoder, bool allow_vcd_target) {
    ((open_vcdiff::VCDiffStreamingDecoder *)decoder)->SetAllowVcdTarget(allow_vcd_target);
}

bool decoder_decode_chunk_to_callbacks(
    void *decoder, const char *data, size_t len,
    void *callback_pointer,
    output_callback_append_fn append_fn,
    output_callback_clear_fn clear_fn,
    output_callback_reserve_fn reserve_fn,
    output_callback_size_fn size_fn
    )
{
    OutputCallbackInterface output(callback_pointer, append_fn, clear_fn, reserve_fn, size_fn);
    return ((open_vcdiff::VCDiffStreamingDecoder *)decoder)->DecodeChunkToInterface(data, len, &output);
}

bool decoder_finish_decoding(void *decoder) {
    return ((open_vcdiff::VCDiffStreamingDecoder *)decoder)->FinishDecoding();
}

void delete_decoder(void *decoder) {
    delete ((open_vcdiff::VCDiffStreamingDecoder *)decoder);
}



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

void vcdiff_free_data(uint8_t *data) {
    free(data);
}

}

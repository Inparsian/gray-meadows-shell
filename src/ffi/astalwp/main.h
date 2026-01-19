#pragma once
#include <astal/wireplumber/wp.h>
#include <gray-meadows-shell/src/ffi/astalwp.rs.h>
#include <rust/cxx.h>
#include "event.h"

struct Node;
struct Endpoint;

class GrayPlumber {
    public:
        AstalWpWp *wp;

        GrayPlumber() :
            wp(astal_wp_wp_get_default()) {}
    
        // Disable copy constructor and assignment operator
        GrayPlumber(const GrayPlumber&) = delete;
        GrayPlumber& operator=(const GrayPlumber&) = delete;
};

// Methods
void init();

rust::String node_get_description(rust::i32 id);
rust::String node_get_icon(rust::i32 id);
bool node_get_mute(rust::i32 id);
rust::String node_get_name(rust::i32 id);
rust::String node_get_path(rust::i32 id);
rust::i32 node_get_serial(rust::i32 id);
rust::f32 node_get_volume(rust::i32 id);
void node_set_mute(rust::i32 id, bool mute);
void node_set_volume(rust::i32 id, rust::f32 volume);

bool endpoint_get_is_default(rust::i32 id);
void endpoint_set_is_default(rust::i32 id, bool is_default);

// Extern Rust methods
void receive_update_node(rust::i32 id, rust::String &property_name) noexcept;
void receive_update_microphone(rust::i32 id, rust::String &property_name) noexcept;
void receive_update_speaker(rust::i32 id, rust::String &property_name) noexcept;
void receive_create_stream(Node node) noexcept;
void receive_remove_stream(Node node) noexcept;
void receive_create_recorder(Node node) noexcept;
void receive_remove_recorder(Node node) noexcept;
void receive_create_microphone(Endpoint endpoint) noexcept;
void receive_remove_microphone(Endpoint endpoint) noexcept;
void receive_create_speaker(Endpoint endpoint) noexcept;
void receive_remove_speaker(Endpoint endpoint) noexcept;
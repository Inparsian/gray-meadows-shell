#include <memory>
#include <mutex>
#include "main.h"

static std::unique_ptr<GrayPlumber> instance = nullptr;
static std::mutex node_operations_mutex;

static AstalWpNode* get_node_by_id(rust::i32 id) {
    if (!instance)
        return nullptr;

    return astal_wp_audio_get_node(astal_wp_wp_get_audio(instance->wp), id);
}

static AstalWpEndpoint* get_speaker_by_id(rust::i32 id) {
    if (!instance)
        return nullptr;

    return astal_wp_audio_get_speaker(astal_wp_wp_get_audio(instance->wp), id);
}

static AstalWpEndpoint* get_microphone_by_id(rust::i32 id) {
    if (!instance)
        return nullptr;

    return astal_wp_audio_get_microphone(astal_wp_wp_get_audio(instance->wp), id);
}

void init() {
    if (!instance) {
        GrayPlumber *plumber = new GrayPlumber();

        plumber->wp
            ? instance = std::unique_ptr<GrayPlumber>(plumber)
            : throw std::runtime_error("Failed to initialize WirePlumber");

        AstalWpAudio *audio = astal_wp_wp_get_audio(instance->wp);

        g_signal_connect(audio, "stream-added", G_CALLBACK(on_stream_added), nullptr);
        g_signal_connect(audio, "stream-removed", G_CALLBACK(on_stream_removed), nullptr);
        g_signal_connect(audio, "recorder-added", G_CALLBACK(on_recorder_added), nullptr);
        g_signal_connect(audio, "recorder-removed", G_CALLBACK(on_recorder_removed), nullptr);
        g_signal_connect(audio, "microphone-added", G_CALLBACK(on_microphone_added), nullptr);
        g_signal_connect(audio, "microphone-removed", G_CALLBACK(on_microphone_removed), nullptr);
        g_signal_connect(audio, "speaker-added", G_CALLBACK(on_speaker_added), nullptr);
        g_signal_connect(audio, "speaker-removed", G_CALLBACK(on_speaker_removed), nullptr);

        g_main_loop_run(g_main_loop_new(nullptr, FALSE));
    }
}

rust::String node_get_description(rust::i32 id) {
    AstalWpNode *node = get_node_by_id(id);

    return node
        ? rust::String(astal_wp_node_get_description(node))
        : rust::String();
}

rust::String node_get_icon(rust::i32 id) {
    AstalWpNode *node = get_node_by_id(id);

    return node
        ? rust::String(astal_wp_node_get_icon(node))
        : rust::String();
}

rust::i32 node_get_id(rust::i32 id) {
    AstalWpNode *node = get_node_by_id(id);

    return node
        ? rust::i32(astal_wp_node_get_id(node))
        : 0;
}

bool node_get_mute(rust::i32 id) {
    AstalWpNode *node = get_node_by_id(id);

    return node 
        ? astal_wp_node_get_mute(node) 
        : false;
}

rust::String node_get_name(rust::i32 id) {
    AstalWpNode *node = get_node_by_id(id);

    return node
        ? rust::String(astal_wp_node_get_name(node))
        : rust::String();
}

rust::String node_get_path(rust::i32 id) {
    AstalWpNode *node = get_node_by_id(id);

    if (!node) {
        return rust::String();
    }

    const char *path = astal_wp_node_get_path(node);
    return path
        ? rust::String(path)
        : rust::String();
}

rust::i32 node_get_serial(rust::i32 id) {
    AstalWpNode *node = get_node_by_id(id);

    return node
        ? rust::i32(astal_wp_node_get_serial(node))
        : 0;
}

rust::f32 node_get_volume(rust::i32 id) {
    AstalWpNode *node = get_node_by_id(id);

    return node
        ? rust::f32(astal_wp_node_get_volume(node))
        : 0.0f;
}

void node_set_mute(rust::i32 id, bool mute) {
    std::lock_guard<std::mutex> lock(node_operations_mutex);

    if (AstalWpNode *node = get_node_by_id(id)) {
        astal_wp_node_set_mute(node, mute);
    }
}

void node_set_volume(rust::i32 id, rust::f32 volume) {
    std::lock_guard<std::mutex> lock(node_operations_mutex);
    
    if (AstalWpNode *node = get_node_by_id(id)) {
        astal_wp_node_set_volume(node, volume);
    }
}

bool endpoint_get_is_default(rust::i32 id) {
    if (AstalWpEndpoint *endpoint = get_speaker_by_id(id)) {
        return astal_wp_endpoint_get_is_default(endpoint);
    }

    if (AstalWpEndpoint *endpoint = get_microphone_by_id(id)) {
        return astal_wp_endpoint_get_is_default(endpoint);
    }
    
    return false;
}

void endpoint_set_is_default(rust::i32 id, bool is_default) {
    std::lock_guard<std::mutex> lock(node_operations_mutex);

    if (AstalWpEndpoint *endpoint = get_speaker_by_id(id)) {
        astal_wp_endpoint_set_is_default(endpoint, is_default);
    } else if (AstalWpEndpoint *endpoint = get_microphone_by_id(id)) {
        astal_wp_endpoint_set_is_default(endpoint, is_default);
    }
}
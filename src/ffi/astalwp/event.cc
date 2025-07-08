#include <event.h>

static std::unordered_map<AstalWpNode*, gulong> node_signals;
static std::unordered_map<AstalWpEndpoint*, gulong> endpoint_signals;

void on_node_notify(AstalWpNode *node, GParamSpec *pspec, gpointer /*user_data*/) {
    receive_update_node(
        astal_wp_node_get_id(node),
        pspec->name
    );
}

void on_microphone_notify(AstalWpEndpoint *endpoint, GParamSpec *pspec, gpointer /*user_data*/) {
    receive_update_microphone(
        astal_wp_node_get_id(ASTAL_WP_NODE(endpoint)),
        pspec->name
    );
}

void on_speaker_notify(AstalWpEndpoint *endpoint, GParamSpec *pspec, gpointer /*user_data*/) {
    receive_update_speaker(
        astal_wp_node_get_id(ASTAL_WP_NODE(endpoint)),
        pspec->name
    );
}

void on_stream_added(AstalWpAudio */*audio*/, AstalWpStream *stream, gpointer /*user_data*/) {
    AstalWpNode *node = ASTAL_WP_NODE(stream);
    Node node_data = make_node_data_from_node(node);

    receive_create_stream(node_data);

    gulong handler_id = g_signal_connect(node, "notify", G_CALLBACK(on_node_notify), nullptr);
    node_signals[node] = handler_id;
}

void on_stream_removed(AstalWpAudio */*audio*/, AstalWpStream *stream, gpointer /*user_data*/) {
    AstalWpNode *node = ASTAL_WP_NODE(stream);
    Node node_data = make_node_data_from_node(node);

    receive_remove_stream(node_data);

    auto it = node_signals.find(node);
    if (it != node_signals.end()) {
        g_signal_handler_disconnect(node, it->second);
        node_signals.erase(it);
    }
}

void on_microphone_added(AstalWpAudio */*audio*/, AstalWpEndpoint *endpoint, gpointer /*user_data*/) {
    Endpoint endpoint_data = make_endpoint_data_from_endpoint(endpoint, EndpointType::Microphone);

    receive_create_microphone(endpoint_data);

    gulong handler_id = g_signal_connect(endpoint, "notify", G_CALLBACK(on_microphone_notify), nullptr);
    endpoint_signals[endpoint] = handler_id;
}

void on_microphone_removed(AstalWpAudio */*audio*/, AstalWpEndpoint *endpoint, gpointer /*user_data*/) {
    Endpoint endpoint_data = make_endpoint_data_from_endpoint(endpoint, EndpointType::Microphone);

    receive_remove_microphone(endpoint_data);

    auto it = endpoint_signals.find(endpoint);
    if (it != endpoint_signals.end()) {
        g_signal_handler_disconnect(endpoint, it->second);
        endpoint_signals.erase(it);
    }
}

void on_speaker_added(AstalWpAudio */*audio*/, AstalWpEndpoint *endpoint, gpointer /*user_data*/) {
    Endpoint endpoint_data = make_endpoint_data_from_endpoint(endpoint, EndpointType::Speaker);

    receive_create_speaker(endpoint_data);

    gulong handler_id = g_signal_connect(endpoint, "notify", G_CALLBACK(on_speaker_notify), nullptr);
    endpoint_signals[endpoint] = handler_id;
}

void on_speaker_removed(AstalWpAudio */*audio*/, AstalWpEndpoint *endpoint, gpointer /*user_data*/) {
    Endpoint endpoint_data = make_endpoint_data_from_endpoint(endpoint, EndpointType::Speaker);

    receive_remove_speaker(endpoint_data);

    auto it = endpoint_signals.find(endpoint);
    if (it != endpoint_signals.end()) {
        g_signal_handler_disconnect(endpoint, it->second);
        endpoint_signals.erase(it);
    }
}
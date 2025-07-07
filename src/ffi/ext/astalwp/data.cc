#include <data.h>

Node make_node_data_from_node(AstalWpNode *node) {
    Node data;

    data.description = rust::String(astal_wp_node_get_description(node));
    data.icon = rust::String(astal_wp_node_get_icon(node));
    data.id = astal_wp_node_get_id(node);
    data.mute = astal_wp_node_get_mute(node);
    data.serial = astal_wp_node_get_serial(node);
    data.volume = astal_wp_node_get_volume(node);

    if (astal_wp_node_get_name(node) != nullptr) {
        data.name = rust::String(astal_wp_node_get_name(node));
    }
    
    if (astal_wp_node_get_path(node) != nullptr) { 
        data.path = rust::String(astal_wp_node_get_path(node));
    }

    return data;
}

Endpoint make_endpoint_data_from_endpoint(AstalWpEndpoint *endpoint) {
    Endpoint data;

    data.is_default = astal_wp_endpoint_get_is_default(endpoint);
    data.node = make_node_data_from_node(ASTAL_WP_NODE(endpoint));

    return data;
}
#pragma once
#include <main.h>

enum EndpointType;
struct Node;
struct Endpoint;

Node make_node_data_from_node(AstalWpNode *node);
Endpoint make_endpoint_data_from_endpoint(AstalWpEndpoint *endpoint);
Endpoint make_endpoint_data_from_endpoint(AstalWpEndpoint *endpoint, EndpointType type);
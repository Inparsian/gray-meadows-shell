#pragma once
#include <main.h>

struct Node;
struct Endpoint;

Node make_node_data_from_node(AstalWpNode *node);
Endpoint make_endpoint_data_from_endpoint(AstalWpEndpoint *endpoint);
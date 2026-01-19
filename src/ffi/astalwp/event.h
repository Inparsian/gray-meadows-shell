#pragma once
#include "main.h"
#include "data.h"
#include <unordered_map>

enum EndpointType;
struct Node;
struct Endpoint;

void on_node_notify(AstalWpNode *node, GParamSpec *pspec, gpointer user_data);
void on_microphone_notify(AstalWpEndpoint *endpoint, GParamSpec *pspec, gpointer user_data);
void on_speaker_notify(AstalWpEndpoint *endpoint, GParamSpec *pspec, gpointer user_data);
void on_stream_added(AstalWpAudio *audio, AstalWpStream *stream, gpointer user_data);
void on_stream_removed(AstalWpAudio *audio, AstalWpStream *stream, gpointer user_data);
void on_recorder_added(AstalWpAudio *audio, AstalWpStream *stream, gpointer user_data);
void on_recorder_removed(AstalWpAudio *audio, AstalWpStream *stream, gpointer user_data);
void on_microphone_added(AstalWpAudio *audio, AstalWpEndpoint *endpoint, gpointer user_data);
void on_microphone_removed(AstalWpAudio *audio, AstalWpEndpoint *endpoint, gpointer user_data);
void on_speaker_added(AstalWpAudio *audio, AstalWpEndpoint *endpoint, gpointer user_data);
void on_speaker_removed(AstalWpAudio *audio, AstalWpEndpoint *endpoint, gpointer user_data);
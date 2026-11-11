#include <astal-cava.h>
#include <glib.h>
#include <gray-meadows-shell/src/ffi/astalcava.rs.h>
#include "main.h"

void init_cava() {
    AstalCavaCava *cava = astal_cava_cava_get_default();
    if (!cava) {
        return;
    }

    astal_cava_cava_set_bars(cava, 50);
    astal_cava_cava_set_framerate(cava, 60);
    astal_cava_cava_set_active(cava, TRUE);
}

rust::Vec<double> get_values() {
    rust::Vec<double> result;
    AstalCavaCava *cava = astal_cava_cava_get_default();
    if (!cava) {
        return result;
    }

    GArray *array = astal_cava_cava_get_values(cava);
    if (!array) {
        return result;
    }

    const guint element_size = g_array_get_element_size(array);
    const auto *data = reinterpret_cast<const guint8 *>(array->data);

    if (element_size == sizeof(double)) {
        const auto *samples = reinterpret_cast<const double *>(data);
        for (guint i = 0; i < array->len; ++i) {
            result.push_back(samples[i]);
        }
    } else if (element_size == sizeof(float)) {
        const auto *samples = reinterpret_cast<const float *>(data);
        for (guint i = 0; i < array->len; ++i) {
            result.push_back(samples[i]);
        }
    }

    return result;
}

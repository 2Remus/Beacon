//
// Created by Adafa Ralph on 6/21/26.
//

#ifndef BEACON2_0_CONTAINER_H
#define BEACON2_0_CONTAINER_H
#include <string>
#include <cstdint>
#include "../servers/serverEntity.h"

class Container {
public:
    std::string server_id;
    std::string jar_path;
    std::int32_t port;
};

std::string create_container(Container& container, const std::string& data_dir, bool online_mode, const std::string& version_path);
#endif //BEACON2_0_CONTAINER_H
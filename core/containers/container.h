//
// Created by Adafa Ralph on 6/21/26.
//

#ifndef BEACON2_0_CONTAINER_H
#define BEACON2_0_CONTAINER_H
#include <string>
#include <cstdint> // Added for std::int32_t
#include "../servers/serverEntity.h"

class Container {
public:
    std::string server_id;
    std::string jar_path;
    std::int32_t port;
};
//wont take ServerEntity???
void create_container(Container& container, std::string data_dir);
#endif //BEACON2_0_CONTAINER_H
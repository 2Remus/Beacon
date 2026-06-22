//
// Created by Adafa Ralph on 6/21/26.
//

#include <iostream>
#include <ostream>
#include "serverEntity.h"
#include "../utils/utils.h"
#include "../containers/container.h"

void add_server(
std::string& id,
std::string& name,
Provider provider,
std::string& version,
std::int32_t ram,
std::int32_t port,
bool online_mode,
std::string& data_dir
) {

    //get the jar path
        // download the jar if it doesnt exist or use it from cache if it does
    // create a container (folder server instance)
    //add server to json db
    std::string resource_dir = get_resource_path(data_dir);
    std::string jar_path = ensure_jar_exists(resource_dir);

    Container container {
        id,
        jar_path,
        port,
    };

    create_container(container);



    // serverEntity server {
    //     id,
    //     name,
    //     jar_path,
    //     version,
    //     "offline",
    //     provider,
    //     std::nullopt,
    //     online_mode,
    //     ram,
    // };





}



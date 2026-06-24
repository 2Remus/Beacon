//
// Created by Adafa Ralph on 6/21/26.
//

#include <iostream>
#include <ostream>
#include "serverEntity.h"
#include "../utils/utils.h"
#include "../containers/container.h"
#include "nlohmann/json.hpp"
#include <fstream>

namespace fs = std::filesystem;

std::string db_init(const std::string& data_dir) {
    std::string resource_dir = get_resource_path(data_dir);
    fs::path resource_path(resource_dir);

    //define db file
    fs::path db_path = resource_path / "db.json";

    if (!fs::exists(db_path)) {
        //open a stream
        std::ofstream dbstream(db_path);
        dbstream.close();
    }

    return db_path.string();

};


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
    std::string jar_path = ensure_jar_exists(resource_dir, version, provider);



    Container container {
        id,
        jar_path,
        port,
    };

    std::string jar_link = create_container(container, data_dir,
        online_mode, jar_path);


    serverEntity server {
        id,
        name,
        jar_link,
        version,
        "offline",
        provider,
        std::nullopt,
        online_mode,
        ram,
    };

    //call db init
    std::string db_path = db_init(data_dir);
    std::ofstream dbstream(db_path);

    //turn struct into var json type
    nlohmann::json t = server;

    if (dbstream.is_open()) {
        //write to json
        dbstream << t.dump(4);
        dbstream.close();
    }
}


nlohmann::json get_servers() {
    //get servers
}



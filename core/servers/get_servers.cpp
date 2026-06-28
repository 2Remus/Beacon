//
// Created by Adafa Ralph on 6/24/26.
//

#include <ostream>
#include "serverEntity.h"
#include "../utils/utils.h"
#include "../containers/container.h"
#include "nlohmann/json.hpp"
#include <fstream>
#include <iostream>

namespace fs = std::filesystem;
namespace jlib = nlohmann;

jlib::json get_servers() {
    //get servers
    // std::string db_path_string = get_resource_path();
    // fs::path db_path(db_path_string);
    // db_path /= "db.json";

    fs::path db_path(get_resource_path());
    db_path /= "db.json";



    if (!fs::exists(db_path)) {
        //create path
        fs::create_directories(db_path.parent_path());
        //create path file
        std::ofstream db(db_path, std::ios::binary);
        if (!db.is_open()) {
            std::cerr << "Failed to create DB file " << db_path.string() << std::endl;
        }

        db << "[]";
        db.close();
    }

    std::ifstream filestream(db_path);

    std::vector<serverEntity> servers;

    if (filestream.is_open()) {
        try {
            jlib::json j;
            filestream >> j;

            if (j.is_array()) {
                servers = j.get<std::vector<serverEntity>>();
            }
        }catch (std::exception& e) {

        }
    }


    //convert vec of server entities to json
    jlib::json server_json = servers;

    std::cout << server_json.dump() << std::endl;

    return server_json;

}
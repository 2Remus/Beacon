//
// Created by Adafa Ralph on 6/24/26.
//

#include <ostream>
#include "serverEntity.h"
#include "../utils/utils.h"
#include "../containers/container.h"
#include "nlohmann/json.hpp"
#include <fstream>

namespace fs = std::filesystem;
namespace jlib = nlohmann;

std::vector<serverEntity> get_servers(const std::string& data_dir) {
    //get servers
    std::string db_path_string = get_resource_path(data_dir);
    fs::path db_path(db_path_string);
    db_path /= "db.json";

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

    return servers;

}
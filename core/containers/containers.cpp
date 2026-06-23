//
// Created by Adafa Ralph on 6/22/26.
//


#include <filesystem>
#include <fstream>

#include "container.h"
#include  "../utils/utils.h"
#include "../servers/serverEntity.h"

namespace fs = std::filesystem;
std::string create_container(Container& container, const std::string& data_dir, bool online_mode, const std::string& version_path) {
    //create dirs
    //logs, world,
    //server properties

    std::string container_path = get_resource_path(data_dir);
    fs::path containder_dir(container_path);

    fs::path server_dir = containder_dir / container.server_id;

    if (!fs::exists(server_dir)) fs::create_directories(server_dir);

    //define a server.properties
    fs::path server_properties = server_dir / "server.properties";
    fs::path eula = server_dir / "eula.txt";

    //stream with path to create file
    std::ofstream filestream(server_properties);
    std::ofstream eulastream(eula);

    if (filestream.is_open()) {
        filestream << "max-players= 20";
        filestream << "online-mode=" << online_mode;
        filestream << "gamemode= survival";
        filestream.close();
    }

    if (eulastream.is_open()) {
        eulastream << "eula=true";
        eulastream.close();
    }


    //symlink binary
    fs::path server(version_path);
    fs::create_symlink(version_path, server);


    return server.string();
}

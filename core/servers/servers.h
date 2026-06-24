//
// Created by Adafa Ralph on 6/21/26.
//
#include "nlohmann/json.hpp"


namespace jlib = nlohmann;
#ifndef BEACON2_0_SERVERS_H
#define BEACON2_0_SERVERS_H


void add_server();

jlib::json get_servers(const std::string& data_dir);

#endif //BEACON2_0_SERVERS_H
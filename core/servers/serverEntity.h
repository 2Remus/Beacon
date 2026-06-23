//
// Created by Adafa Ralph on 6/21/26.
//

#ifndef BEACON2_0_SERVERENTITY_H
#define BEACON2_0_SERVERENTITY_H
#include <string>


enum Provider {
    Vanilla,
    Paper,
    Fabric,
    Forge,
};


class serverEntity {
    public:
    std::string id;
    std::string name;
    std::string instance_path;
    std::string version;
    std::string status;
    Provider provider;
    std::optional<std::string> world;
    std::optional<std::int32_t> port;
    std::optional<std::int32_t> ram;
    bool online_mode;

};




#endif //BEACON2_0_SERVERENTITY_H
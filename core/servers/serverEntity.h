//
// Created by Adafa Ralph on 6/21/26.
//

#ifndef BEACON2_0_SERVERENTITY_H
#define BEACON2_0_SERVERENTITY_H
#include <string>
#include "nlohmann/json.hpp"


enum Provider {
    Vanilla,
    Paper,
    Fabric,
    Forge,
};

NLOHMANN_JSON_SERIALIZE_ENUM(Provider,{
    {Vanilla, "Vanilla"},
    {Paper, "Paper"},
    {Fabric, "Fabric"},
    {Forge, "Forge"}
})



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

NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(serverEntity,
    id,
    name,
    instance_path,
    version,
    status,
    provider,
    world,
    port,
    ram,
    online_mode
)


#endif //BEACON2_0_SERVERENTITY_H
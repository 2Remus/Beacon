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

namespace nlohmann {
    template <typename T>
    struct adl_serializer<std::optional<T>> {
        static void to_json(json& j, const std::optional<T>& opt) {
            if (!opt) {
                j = nullptr;
            } else {
                j = *opt; // this will call adl_serializer<T>::to_json
            }
        }

        static void from_json(const json& j, std::optional<T>& opt) {
            if (j.is_null()) {
                opt = std::nullopt;
            } else {
                opt = j.get<T>(); // this will call adl_serializer<T>::from_json
            }
        }
    };
}

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
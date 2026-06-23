//
// Created by Adafa Ralph on 6/21/26.
//

#ifndef BEACON2_0_UTILS_H
#define BEACON2_0_UTILS_H
#include <string_view>
#include <__filesystem/path.h>

std::string ensure_jar_exists(const std::string& path, const std::string& version, Provider provider);

std::filesystem::path get_resource_path(std::string_view path);

#endif //BEACON2_0_UTILS_H

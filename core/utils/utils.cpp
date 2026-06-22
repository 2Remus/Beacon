#include <string_view>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <format>
#include <cpr/cpr.h>
#include <nlohmann/json.hpp>
#include "webview.h"
#include "../servers/serverEntity.h"

namespace fs = std::filesystem;

// Removed empty globals. Better to keep paths localized or passed cleanly.

std::string get_resource_path(const std::string_view path) {
    fs::path my_path(path);
#if defined(_WIN32) || defined(_WIN64)
    my_path /= "resources";
#else
    my_path /= "Resources";
#endif
    return my_path.string();
}

std::string ensure_jar_exists(const std::string& path, const std::string& version, Provider provider) {

    fs::path cache_dir(path);
    cache_dir /= "cache";

    if (!fs::exists(cache_dir)) {
        fs::create_directories(cache_dir);
    }

    // Fix: target_dir should point directly to the full destination file path
    std::string jar_name = std::format("server-{}.jar", version);
    fs::path target_file_path = cache_dir / jar_name;

    // 1. If the JAR already exists locally, immediately return its path!
    if (fs::exists(target_file_path)) {
        return target_file_path.string();
    }

    std::string download_url = "";

    // Determine download URL based on Provider
    switch(provider) {
        case Provider::Vanilla: {
            std::string manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
            cpr::Response response = cpr::Get(cpr::Url{manifest_url});

            if (response.status_code != 200) {
                throw std::runtime_error("Failed to fetch version manifest");
            }

            nlohmann::json manifest = nlohmann::json::parse(response.text);
            std::string version_url = "";

            for (const auto& entry : manifest["versions"]) {
                if (entry.contains("id") && entry["id"] == version) {
                    version_url = entry["url"].get<std::string>();
                    break;
                }
            }

            if (version_url.empty()) {
                throw std::runtime_error("Vanilla version " + version + " not found");
            }

            cpr::Response meta_response = cpr::Get(cpr::Url{version_url});
            if (meta_response.status_code != 200) {
                throw std::runtime_error("Failed to fetch version metadata");
            }

            nlohmann::json meta = nlohmann::json::parse(meta_response.text);

            if (meta.contains("downloads") &&
                meta["downloads"].contains("server") &&
                meta["downloads"]["server"].contains("url")) {
                download_url = meta["downloads"]["server"]["url"].get<std::string>();
            } else {
                throw std::runtime_error("Failed to locate server download URL in metadata");
            }
            break;
        }
        default:
            download_url = "https://mojang.com/" + jar_name;
            break;
    }

    if (!download_url.empty()) {
        std::ofstream download_file(target_file_path, std::ios::binary);
        if (!download_file.is_open()) {
            throw std::runtime_error("Failed to open local file stream for writing download");
        }


        cpr::Response download_res = cpr::Download(download_file, cpr::Url{download_url});

        if (download_res.status_code != 200) {
            fs::remove(target_file_path);
            throw std::runtime_error("Failed downloading server JAR from Mojang");
        }
    }

    return target_file_path.string();
}
cmake_minimum_required(VERSION 3.16)
project(Beacon2_0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# ==============================================================================
# 1. macOS Homebrew Path Fixes (CRITICAL: Must be BEFORE FetchContent)
# ==============================================================================
if(APPLE)
    # Tell CMake's internal find_package routines to look in Homebrew paths
    list(APPEND CMAKE_PREFIX_PATH "/opt/homebrew")

    # Prepend global search directories for any nested targets like Curl
    include_directories(SYSTEM /opt/homebrew/include)
    link_directories(/opt/homebrew/lib)
endif()

# Force internal curl dependencies to explicitly check Homebrew or turn off unneeded features
set(CMAKE_USE_LIBIDN2 OFF CACHE BOOL "Disable IDN2 support for curl" FORCE)

# ==============================================================================
# 2. Dependencies
# ==============================================================================
include(FetchContent)

FetchContent_Declare(
        webview
        GIT_REPOSITORY https://github.com/webview/webview
        GIT_TAG 0.12.0
)
FetchContent_MakeAvailable(webview)

FetchContent_Declare(
        json
        GIT_REPOSITORY https://github.com/nlohmann/json
        GIT_TAG v3.12.0
)
FetchContent_MakeAvailable(json)

set(BUILD_SHARED_LIBS OFF)
set(CPR_BUILD_TESTS OFF CACHE INTERNAL "")

FetchContent_Declare(
        cpr
        GIT_REPOSITORY https://github.com/libcpr/cpr.git
        GIT_TAG 1.11.1
)
FetchContent_MakeAvailable(cpr)

# ==============================================================================
# 3. Main Executable & Build Pipeline
# ==============================================================================
add_executable(Beacon2_0 main.cpp
        core/servers/servers.h
        core/servers/add_server.cpp
        core/containers/container.h
        core/utils/utils.cpp
)

target_include_directories(Beacon2_0 PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/interface2/dist")
target_link_libraries(Beacon2_0 PRIVATE webview::core nlohmann_json::nlohmann_json cpr::cpr)

# Asset Compilation Step
set(GENERATED_UI_HEADER "${CMAKE_CURRENT_SOURCE_DIR}/interface2/dist/index_html.h")

add_custom_command(
        OUTPUT "${GENERATED_UI_HEADER}"
        COMMAND npm run build --prefix "${CMAKE_CURRENT_SOURCE_DIR}/interface2"
        WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
        COMMENT "Building user interface production assets..."
)

target_sources(Beacon2_0 PRIVATE "${GENERATED_UI_HEADER}")
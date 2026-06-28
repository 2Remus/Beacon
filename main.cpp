#include <iostream>
#include <string>
#include "webview.h"
#include "index_html.h"
#include "core/servers/servers.h"

int main() {
    webview::webview main_window(true, nullptr);
    main_window.set_title("Beacon");
    main_window.set_size(1280, 720, WEBVIEW_HINT_NONE);

    const std::string html_content(reinterpret_cast<const char*>(INDEX_HTML_BYTES), INDEX_HTML_SIZE);

    main_window.set_html(html_content);


    try {
        main_window.bind("getServers", [](const std::string& req) -> std::string {
            nlohmann::json servers = get_servers().dump();

            return servers;
        });
    }catch (std::exception& e) {
        std::cerr << e.what() << std::endl;
    }
    main_window.run();
    return 0;
}
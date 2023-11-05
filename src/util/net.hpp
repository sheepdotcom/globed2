#pragma once
#include <string>

namespace util::net {
    // Initialize all networking libraries (calls `WSAStartup` on Windows, does nothing on other platforms)
    void initialize();

    // Cleanup networking resources (calls `WSACleanup` on Windows, does nothing on other platforms)
    void cleanup();

    // Returns the last network error code (calls `WSAGetLastError` on Windows, grabs `errno` on other platforms)
    int lastErrorCode();

    // Grabs the error code from `lastErrorCode` and formats to a string
    std::string lastErrorString();

    // Formats the error code to a string.
    std::string lastErrorString(int code);

    // Throws an exception with the message being the value from `lastErrorString()`
    [[noreturn]] void throwLastError();
}
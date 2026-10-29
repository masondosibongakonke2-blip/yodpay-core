#ifndef TELEMETRY_PARSER_HPP
#define TELEMETRY_PARSER_HPP

#include <string>
#include <vector>
#include <sstream>
#include <stdexcept>
#include <iostream>
#include <cassert>

namespace YodPay {

struct SatelliteTelemetry {
    double latitude;
    double longitude;
    uint32_t timestamp_utc;
    uint8_t network_status; // 0 = Normal, 1 = Partitioned Mesh, 2 = Grid Collapse
};

class TelemetryParser {
public:
    // Parses standard public telemetry strings broadcast by edge/satellite devices
    // Example NMEA-style format: "$YODTE,28.4561,-30.1234,132450,2*4D"
    static SatelliteTelemetry parseFromBroadcast(const std::string& payload) {
        if (payload.rfind("$YODTE", 0) != 0) {
            throw std::invalid_argument("Invalid telemetry identifier header");
        }

        // Strip any trailing checksum separator '*' or network carriage returns
        std::string mainBody = payload.substr(0, payload.find('*'));
        std::vector<std::string> parts;
        std::string token;
        std::stringstream ss(mainBody);

        while (std::getline(ss, token, ',')) {
            parts.push_back(token);
        }

        if (parts.size() < 5) {
            throw std::runtime_error("Incomplete telemetry payload fields");
        }

        try {
            SatelliteTelemetry telemetry;
            telemetry.latitude = std::stod(parts[1]);
            telemetry.longitude = std::stod(parts[2]);
            telemetry.timestamp_utc = static_cast<uint32_t>(std::stoul(parts[3]));
            telemetry.network_status = static_cast<uint8_t>(std::stoul(parts[4]));
            return telemetry;
        } catch (...) {
            throw std::runtime_error("Failed to parse numeric telemetry fields");
        }
    }
};

// Internal unit verification block to prove correctness straight to the compiler
inline void runLocalTelemetryTests() {
    // Test Case 1: Valid packet parsing
    std::string mockPacket = "$YODTE,28.4561,-30.1234,132450,2*4D";
    auto data = TelemetryParser::parseFromBroadcast(mockPacket);
    assert(data.latitude == 28.4561);
    assert(data.longitude == -30.1234);
    assert(data.timestamp_utc == 132450);
    assert(data.network_status == 2);

    // Test Case 2: Reject invalid identifier header
    try {
        TelemetryParser::parseFromBroadcast("$INVALID,28.4561,-30.1234,132450,2");
        assert(false); // Should not reach here
    } catch (const std::invalid_argument&) {
        // Success: Exception correctly thrown
    }

    std::cout << "[C++ Native Validation]: All telemetry parsing verification checks passed.\n";
}

} // namespace YodPay

#endif // TELEMETRY_PARSER_HPP

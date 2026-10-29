use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct SatelliteTelemetry {
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp_utc: u32,
    pub network_status: u8, // 0 = Normal, 1 = Partitioned Mesh, 2 = Grid Collapse / Isolation
}

#[derive(Debug, PartialEq)]
pub enum TelemetryError {
    InvalidHeader,
    IncompletePayload,
    ParseFailure,
}

impl SatelliteTelemetry {
    /// Parses standard public telemetry strings broadcast by edge/satellite devices
    /// Example NMEA-style format: "$YODTE,28.4561,-30.1234,132450,2*4D"
    pub fn parse_from_broadcast(payload: &str) -> Result<Self, TelemetryError> {
        if !payload.starts_with("$YODTE") {
            return Err(TelemetryError::InvalidHeader);
        }

        // Clean up formatting carriage returns or trailing spaces
        let clean_payload = payload.trim_end_matches(|c| c == '\r' || c == '\n');
        
        // Split out the checksum separator '*' if present
        let main_body = clean_payload.split('*').next().unwrap_or(clean_payload);
        let parts: Vec<&str> = main_body.split(',').collect();

        if parts.len() < 5 {
            return Err(TelemetryError::IncompletePayload);
        }

        let lat = parts[1].parse::<f64>().map_err(|_| TelemetryError::ParseFailure)?;
        let lon = parts[2].parse::<f64>().map_err(|_| TelemetryError::ParseFailure)?;
        let time = parts[3].parse::<u32>().map_err(|_| TelemetryError::ParseFailure)?;
        let status = parts[4].parse::<u8>().map_err(|_| TelemetryError::ParseFailure)?;

        Ok(SatelliteTelemetry {
            latitude: lat,
            longitude: lon,
            timestamp_utc: time,
            network_status: status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_satellite_telemetry_parsing() {
        // Simulating a free public broadcast data packet string received on the edge node
        let incoming_packet = "$YODTE,28.4561,-30.1234,132450,2*4D";
        
        let result = SatelliteTelemetry::parse_from_broadcast(incoming_packet);
        
        assert!(result.is_ok());
        let telemetry = result.unwrap();
        
        assert_eq!(telemetry.latitude, 28.4561);
        assert_eq!(telemetry.longitude, -30.1234);
        assert_eq!(telemetry.timestamp_utc, 132450);
        assert_eq!(telemetry.network_status, 2); // Correctly identified as isolated mode
    }

    #[test]
    fn test_invalid_header_rejection() {
        let bad_packet = "$INVALID,28.4561,-30.1234,132450,2";
        let result = SatelliteTelemetry::parse_from_broadcast(bad_packet);
        
        assert_eq!(result, Err(TelemetryError::InvalidHeader));
    }

    #[test]
    fn test_malformed_data_handling() {
        let broken_coordinates = "$YODTE,not_a_latitude,-30.1234,132450,2";
        let result = SatelliteTelemetry::parse_from_broadcast(broken_coordinates);
        
        assert_eq!(result, Err(TelemetryError::ParseFailure));
    }
}

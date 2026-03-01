#ifndef DLE_LEVELREADER_H
#define DLE_LEVELREADER_H

#include <expected>
#include <string>
#include <cstdint>
#include "../types/Types.h"
#include "../mine/Mine.h"

class QDataStream;

namespace dle {

/**
 * @brief Error type for level reading operations
 */
struct LevelReadError {
    std::string message;
    
    explicit LevelReadError(std::string msg) : message(std::move(msg)) {}
};

/**
 * @brief Reader for Descent 1 (RDL) and Descent 2 (RL2) level files
 * 
 * Parses binary level files containing mine geometry, segments, walls,
 * triggers, and objects using Qt's QDataStream for binary I/O.
 * 
 * ## Format Support
 * - **RDL**: Descent 1 Registered Level
 * - **SDL**: Descent 1 Shareware Level (old format)
 * - **RL2**: Descent 2 Level
 * - **SL2**: Descent 2 Shareware Level
 * 
 * ## File Structure
 * 1. Header (compiled version, counts)
 * 2. Vertices (3D points)
 * 3. Segments (cube rooms with 6 sides each)
 * 4. Segment extras (function, properties, lighting)
 * 5. D2X-XL extensions (if present)
 * 
 * @see docs/formats/LEVEL_FORMAT.md for complete format specification
 */
class LevelReader {
public:
    /**
     * @brief Load a level file into a Mine object
     * 
     * @param filename Path to the .rdl, .rl2, .sdl, or .sl2 file
     * @param mine Mine object to populate with level data
     * @return std::expected<void, LevelReadError> Success or error
     */
    static std::expected<void, LevelReadError> load(const std::string& filename, Mine& mine);

private:
    LevelReader() = delete;  // Static class, no instances
    
    // Reader state
    struct ReaderState {
        QDataStream& stream;
        bool newFileFormat;
        uint8_t version;
        
        explicit ReaderState(QDataStream& s) 
            : stream(s), newFileFormat(true), version(20) {}
        
        // Read helper methods
        fix readFix();
        Vector readVector();
        UVLS readUVL();
    };
    
    // Parsing functions (all return std::expected)
    static std::expected<void, LevelReadError> parseHeader(
        ReaderState& reader,
        const std::string& filename,
        uint16_t& vertexCount,
        uint16_t& segmentCount
    );
    
    static std::expected<void, LevelReadError> parseVertices(
        ReaderState& reader,
        std::vector<Vertex>& vertices,
        uint16_t count
    );
    
    static std::expected<void, LevelReadError> parseSegments(
        ReaderState& reader,
        std::vector<Segment>& segments,
        uint16_t count,
        uint16_t vertexCount
    );
    
    static std::expected<void, LevelReadError> parseSegment(
        ReaderState& reader,
        Segment& segment,
        uint16_t vertexCount
    );
    
    static std::expected<void, LevelReadError> parseSegmentChildren(
        ReaderState& reader,
        Segment& segment,
        uint8_t flags
    );
    
    static std::expected<void, LevelReadError> parseSegmentVertices(
        ReaderState& reader,
        Segment& segment,
        uint16_t vertexCount
    );
    
    static std::expected<void, LevelReadError> parseSegmentFunction(
        ReaderState& reader,
        Segment& segment,
        uint8_t flags
    );
    
    static std::expected<void, LevelReadError> parseSide(
        ReaderState& reader,
        Side& side,
        int16_t child
    );
    
    static std::expected<void, LevelReadError> parseSegmentExtras(
        ReaderState& reader,
        std::vector<Segment>& segments
    );
    
    static std::expected<void, LevelReadError> parseSegmentExtra(
        ReaderState& reader,
        Segment& segment
    );
    
    // Helper functions
    static bool isOldFileFormat(const std::string& filename);
    static uint8_t inferVersion(bool newFileFormat);
};

} // namespace dle

#endif // DLE_LEVELREADER_H

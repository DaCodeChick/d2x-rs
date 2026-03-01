#include "LevelReader.h"
#include <QFile>
#include <QDataStream>
#include <algorithm>
#include <format>
#include <print>

namespace dle {

// ================================================================================================
// CONSTANTS
// ================================================================================================

constexpr uint8_t COMPILED_MINE_VERSION = 0;
constexpr uint8_t MINE_VERSION = 20;
constexpr uint8_t LEVEL_VERSION_D2_SHAREWARE = 5;

// ================================================================================================
// ReaderState IMPLEMENTATION
// ================================================================================================

fix LevelReader::ReaderState::readFix() {
    int32_t value;
    stream >> value;
    return value;
}

Vector LevelReader::ReaderState::readVector() {
    fix x, y, z;
    stream >> x >> y >> z;
    return Vector{x, y, z};
}

UVLS LevelReader::ReaderState::readUVL() {
    // UVL stored as i16 (u, v) + u16 (l)
    // Scale: u/v << 5 (multiply by 32), l << 1 (multiply by 2)
    int16_t u_raw, v_raw;
    uint16_t l_raw;
    
    stream >> u_raw >> v_raw >> l_raw;
    
    return UVLS{
        static_cast<fix>(static_cast<int32_t>(u_raw) << 5),
        static_cast<fix>(static_cast<int32_t>(v_raw) << 5),
        static_cast<uint16_t>(l_raw << 1)
    };
}

// ================================================================================================
// PUBLIC API
// ================================================================================================

std::expected<void, LevelReadError> LevelReader::load(const std::string& filename, Mine& mine) {
    QFile file(QString::fromStdString(filename));
    if (!file.open(QIODevice::ReadOnly)) {
        return std::unexpected(LevelReadError{
            std::format("Failed to open file: {}", filename)
        });
    }
    
    QDataStream stream(&file);
    stream.setByteOrder(QDataStream::LittleEndian);
    
    // Initialize reader state
    ReaderState reader(stream);
    
    // Parse header
    uint16_t vertexCount = 0;
    uint16_t segmentCount = 0;
    if (auto result = parseHeader(reader, filename, vertexCount, segmentCount); !result) {
        return result;
    }
    
    std::println("Parsing level: {} vertices, {} segments (version {}, {} format)",
                 vertexCount, segmentCount, reader.version,
                 reader.newFileFormat ? "new" : "old");
    
    // Clear mine and prepare for new data
    mine.clear();
    
    // Parse vertices
    std::vector<Vertex> vertices;
    if (auto result = parseVertices(reader, vertices, vertexCount); !result) {
        return result;
    }
    
    // Parse segments
    std::vector<Segment> segments;
    if (auto result = parseSegments(reader, segments, segmentCount, vertexCount); !result) {
        return result;
    }
    
    // Parse segment extras (D2+ only, version >= 2)
    if (reader.version >= 2) {
        if (auto result = parseSegmentExtras(reader, segments); !result) {
            return result;
        }
    }
    
    // Populate mine
    mine.getVertices() = std::move(vertices);
    mine.getSegments() = std::move(segments);
    
    // Set file type based on version
    if (reader.version == LEVEL_VERSION_D1) {
        mine.setFileType(FileType::RDL);
    } else if (reader.version >= 9) {
        mine.setFileType(FileType::D2X_XL);
    } else {
        mine.setFileType(FileType::RL2);
    }
    mine.setLevelVersion(reader.version);
    
    std::println("Successfully parsed level: {} vertices, {} segments",
                 mine.getVertexCount(), mine.getSegmentCount());
    
    return {};
}

// ================================================================================================
// PARSING FUNCTIONS
// ================================================================================================

std::expected<void, LevelReadError> LevelReader::parseHeader(
    ReaderState& reader,
    const std::string& filename,
    uint16_t& vertexCount,
    uint16_t& segmentCount
) {
    // Detect file format from extension
    reader.newFileFormat = !isOldFileFormat(filename);
    
    // Read compiled version (always 0)
    uint8_t compiled_version;
    reader.stream >> compiled_version;
    
    if (compiled_version != COMPILED_MINE_VERSION) {
        std::println("Warning: Unexpected compiled version {} (expected 0)", compiled_version);
    }
    
    // Read counts (format depends on old/new)
    if (reader.newFileFormat) {
        reader.stream >> vertexCount >> segmentCount;
    } else {
        // Old format uses i32
        int32_t vc, sc;
        reader.stream >> vc >> sc;
        vertexCount = static_cast<uint16_t>(vc);
        segmentCount = static_cast<uint16_t>(sc);
    }
    
    // Infer version
    reader.version = inferVersion(reader.newFileFormat);
    
    return {};
}

std::expected<void, LevelReadError> LevelReader::parseVertices(
    ReaderState& reader,
    std::vector<Vertex>& vertices,
    uint16_t count
) {
    vertices.reserve(count);
    
    for (uint16_t i = 0; i < count; ++i) {
        Vector pos = reader.readVector();
        vertices.push_back(Vertex{pos});
    }
    
    return {};
}

std::expected<void, LevelReadError> LevelReader::parseSegments(
    ReaderState& reader,
    std::vector<Segment>& segments,
    uint16_t count,
    uint16_t vertexCount
) {
    segments.reserve(count);
    
    for (uint16_t i = 0; i < count; ++i) {
        Segment segment;
        if (auto result = parseSegment(reader, segment, vertexCount); !result) {
            return std::unexpected(LevelReadError{
                std::format("Failed to parse segment {}: {}", i, result.error().message)
            });
        }
        segments.push_back(std::move(segment));
    }
    
    return {};
}

std::expected<void, LevelReadError> LevelReader::parseSegment(
    ReaderState& reader,
    Segment& segment,
    uint16_t vertexCount
) {
    // Read flags byte (new format) or use 0x7F (old format)
    uint8_t flags = reader.newFileFormat ? ([&]{ uint8_t f; reader.stream >> f; return f; })() : 0x7F;
    
    // Parse in version-specific order
    if (reader.version == LEVEL_VERSION_D2_SHAREWARE) {
        // D2 Shareware (v5): function, verts, children
        if (auto result = parseSegmentFunction(reader, segment, flags); !result) {
            return result;
        }
        if (auto result = parseSegmentVertices(reader, segment, vertexCount); !result) {
            return result;
        }
        if (auto result = parseSegmentChildren(reader, segment, flags); !result) {
            return result;
        }
    } else if (reader.version == LEVEL_VERSION_D1) {
        // D1 (v1): children, verts, function (with static light)
        if (auto result = parseSegmentChildren(reader, segment, flags); !result) {
            return result;
        }
        if (auto result = parseSegmentVertices(reader, segment, vertexCount); !result) {
            return result;
        }
        
        // D1 format: i16 static_light + u8 function
        int16_t static_light;
        uint8_t func;
        reader.stream >> static_light >> func;
        
        segment.setStaticLight(static_cast<int>(static_light));
        segment.setFunction(static_cast<SegmentFunction>(func));
    } else {
        // D2+ (v2-20): children, verts (no inline function)
        if (auto result = parseSegmentChildren(reader, segment, flags); !result) {
            return result;
        }
        if (auto result = parseSegmentVertices(reader, segment, vertexCount); !result) {
            return result;
        }
    }
    
    // Read wall flags (which sides have walls)
    uint8_t wall_flags = reader.newFileFormat ? ([&]{ uint8_t f; reader.stream >> f; return f; })() : 0x3F;
    
    // Read wall numbers for each side
    for (int i = 0; i < NUM_SIDES; ++i) {
        uint16_t wall_num = 0xFFFF;
        if ((wall_flags & (1 << i)) != 0) {
            reader.stream >> wall_num;
        }
        segment.getSide(i).setWall(wall_num);
    }
    
    // Read sides
    for (int i = 0; i < NUM_SIDES; ++i) {
        int16_t child = segment.getSide(i).getChild();
        if (auto result = parseSide(reader, segment.getSide(i), child); !result) {
            return std::unexpected(LevelReadError{
                std::format("Failed to parse side {}: {}", i, result.error().message)
            });
        }
    }
    
    return {};
}

std::expected<void, LevelReadError> LevelReader::parseSegmentChildren(
    ReaderState& reader,
    Segment& segment,
    uint8_t flags
) {
    for (int i = 0; i < NUM_SIDES; ++i) {
        int16_t child;
        if ((flags & (1 << i)) != 0) {
            reader.stream >> child;
        } else {
            child = -1;
        }
        segment.getSide(i).setChild(child);
    }
    return {};
}

std::expected<void, LevelReadError> LevelReader::parseSegmentVertices(
    ReaderState& reader,
    Segment& segment,
    uint16_t vertexCount
) {
    for (int i = 0; i < NUM_VERTICES_PER_SEGMENT; ++i) {
        uint16_t vertex_idx;
        reader.stream >> vertex_idx;
        
        if (vertex_idx >= vertexCount) {
            return std::unexpected(LevelReadError{
                std::format("Vertex index {} out of range (max {})", vertex_idx, vertexCount)
            });
        }
        segment.setVertexId(i, vertex_idx);
    }
    return {};
}

std::expected<void, LevelReadError> LevelReader::parseSegmentFunction(
    ReaderState& reader,
    Segment& segment,
    uint8_t flags
) {
    // V5 format: function data is present if bit 6 of flags is set
    if ((flags & (1 << 6)) != 0) {
        uint8_t func;
        reader.stream >> func;
        segment.setFunction(static_cast<SegmentFunction>(func));
    }
    return {};
}

std::expected<void, LevelReadError> LevelReader::parseSide(
    ReaderState& reader,
    Side& side,
    int16_t child
) {
    // Determine if this side has textures
    bool is_solid = (child == -1);
    bool has_wall = side.hasWall();
    bool has_texture = is_solid || has_wall;
    
    // Read corner indices (v25+ only)
    if (reader.version >= 25) {
        for (int i = 0; i < NUM_VERTICES_PER_SIDE; ++i) {
            uint8_t corner_idx;
            reader.stream >> corner_idx;
            side.setVertexIdIndex(i, corner_idx);
        }
    }
    
    if (!has_texture) {
        // No texture data
        side.setBaseTexture(0);
        side.setOverlayTexture(0);
        return {};
    }
    
    // Read base texture
    int16_t base_tex_raw;
    if (reader.newFileFormat) {
        uint16_t temp;
        reader.stream >> temp;
        base_tex_raw = static_cast<int16_t>(temp);
    } else {
        reader.stream >> base_tex_raw;
    }
    
    side.setBaseTexture(static_cast<int16_t>(base_tex_raw & 0x7FFF));
    
    // Check if overlay is present
    bool has_overlay = reader.newFileFormat ? ((base_tex_raw & 0x8000) != 0) : true;
    
    if (has_overlay) {
        int16_t ovl_tex_raw;
        reader.stream >> ovl_tex_raw;
        side.setOverlayTexture(ovl_tex_raw);  // Includes alignment in upper bits
    } else {
        side.setOverlayTexture(0);
    }
    
    // Read UVLs (4 corners)
    for (int i = 0; i < NUM_VERTICES_PER_SIDE; ++i) {
        side.setUVL(i, reader.readUVL());
    }
    
    return {};
}

std::expected<void, LevelReadError> LevelReader::parseSegmentExtras(
    ReaderState& reader,
    std::vector<Segment>& segments
) {
    for (auto& segment : segments) {
        if (auto result = parseSegmentExtra(reader, segment); !result) {
            return result;
        }
    }
    return {};
}

std::expected<void, LevelReadError> LevelReader::parseSegmentExtra(
    ReaderState& reader,
    Segment& segment
) {
    // Read function type
    uint8_t function_raw;
    reader.stream >> function_raw;
    
    // Read obj_producer and value
    int16_t obj_producer;
    int16_t value;
    if (reader.version < 24) {
        uint8_t prod_u8;
        int8_t val_i8;
        reader.stream >> prod_u8 >> val_i8;
        obj_producer = static_cast<int16_t>(prod_u8);
        value = static_cast<int16_t>(val_i8);
    } else {
        reader.stream >> obj_producer >> value;
    }
    
    // Read flags (unused but must be read)
    uint8_t _flags;
    reader.stream >> _flags;
    
    // Read props and damage
    if (reader.version <= 20) {
        // Old format: upgrade from function type
        segment.setFunction(static_cast<SegmentFunction>(function_raw));
        segment.setProducerId(obj_producer);
        segment.setValue(value);
    } else {
        // New format: explicit props and damage
        uint8_t props;
        int16_t damage0, damage1;
        reader.stream >> props >> damage0 >> damage1;
        
        segment.setFunction(static_cast<SegmentFunction>(function_raw));
        segment.setProducerId(obj_producer);
        segment.setValue(value);
    }
    
    // Read average segment light
    fix avg_seg_light = reader.readFix();
    segment.setStaticLight(fixToInt(avg_seg_light));
    
    return {};
}

// ================================================================================================
// HELPER FUNCTIONS
// ================================================================================================

bool LevelReader::isOldFileFormat(const std::string& filename) {
    // Old format = D1 shareware (.sdl or .SDL)
    if (filename.length() < 4) {
        return false;
    }
    
    std::string ext = filename.substr(filename.length() - 4);
    std::transform(ext.begin(), ext.end(), ext.begin(), ::tolower);
    
    return ext == ".sdl";
}

uint8_t LevelReader::inferVersion(bool newFileFormat) {
    // Default to current version for new format, D1 for old format
    return newFileFormat ? MINE_VERSION : LEVEL_VERSION_D1;
}

} // namespace dle

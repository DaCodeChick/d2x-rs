#pragma once

#include "../types/Types.h"
#include "Side.h"
#include <array>
#include <memory>

class QDataStream;  // Qt forward declaration

namespace dle {

// Side vertex indices (which vertices of the segment form each side)
constexpr uint8_t SIDE_VERTEX_TABLE[6][4] = {
    {7, 6, 2, 3},  // Right
    {0, 4, 7, 3},  // Top
    {0, 1, 5, 4},  // Front
    {4, 5, 1, 0},  // Left (corrected, was backwards)
    {1, 2, 6, 5},  // Bottom
    {3, 2, 1, 0}   // Back
};

// Opposite side table
constexpr uint8_t OPPOSITE_SIDE_TABLE[6] = {
    3,  // Right <-> Left
    4,  // Top <-> Bottom
    5,  // Front <-> Back
    0,  // Left <-> Right
    1,  // Bottom <-> Top
    2   // Back <-> Front
};

// Edge vertex table (12 edges, 2 vertices each)
constexpr uint8_t EDGE_VERTEX_TABLE[12][2] = {
    {0, 1}, {1, 2}, {2, 3}, {3, 0},  // Front face edges
    {4, 5}, {5, 6}, {6, 7}, {7, 4},  // Back face edges
    {0, 4}, {1, 5}, {2, 6}, {3, 7}   // Connecting edges
};

// Segment functions (what the segment is used for)
enum class SegmentFunction : uint8_t {
    NONE = 0,
    ENERGY_CENTER = 1,    // Energy center (repair center)
    REPAIR_CENTER = 2,    // Repair center (energy center - same as above in D1)
    REACTOR = 3,          // Reactor (control center)
    ROBOT_MAKER = 4,      // Robot matcen
    GOAL_BLUE = 5,        // Blue goal (CTF)
    GOAL_RED = 6,         // Red goal (CTF)
    TEAM_BLUE = 7,        // Blue team area
    TEAM_RED = 8,         // Red team area
    SPEED_BOOST = 9,      // Speed boost
    SKYBOX = 10,          // Skybox
    EQUIP_MAKER = 11      // Equipment maker (powerup matcen)
};

// Segment properties (flags for special behaviors)
enum class SegmentProperty : uint8_t {
    NONE = 0,
    WATER = 1,            // Water segment (physics change)
    LAVA = 2,             // Lava segment (damage)
    BLOCKED = 4,          // Blocked (no robots can enter)
    NO_DAMAGE = 8,        // No damage (safe zone)
    SELF_ILLUMINATE = 16, // Self-illuminated
    LIGHT_FOG = 32,       // Light fog
    DENSE_FOG = 64        // Dense fog
};

// Combine multiple properties
inline SegmentProperty operator|(SegmentProperty a, SegmentProperty b) {
    return static_cast<SegmentProperty>(static_cast<uint8_t>(a) | static_cast<uint8_t>(b));
}

inline bool operator&(SegmentProperty a, SegmentProperty b) {
    return (static_cast<uint8_t>(a) & static_cast<uint8_t>(b)) != 0;
}

/**
 * @brief Segment represents a cube in the mine
 * 
 * Each segment has:
 * - 8 vertices (corner points)
 * - 6 sides (faces)
 * - Special function (matcen, goal, etc.)
 * - Properties (water, lava, etc.)
 * - Static lighting value
 */
class Segment {
public:
    Segment();
    ~Segment() = default;
    
    // Copy/move constructors
    Segment(const Segment&) = default;
    Segment& operator=(const Segment&) = default;
    Segment(Segment&&) = default;
    Segment& operator=(Segment&&) = default;
    
    // Initialization
    void clear();
    void setup();
    void reset(int sideIndex = -1);
    
    // Vertices
    uint16_t getVertexId(int index) const { return m_vertexIds[index]; }
    void setVertexId(int index, uint16_t vertexId) { m_vertexIds[index] = vertexId; }
    const std::array<uint16_t, NUM_VERTICES_PER_SEGMENT>& getVertexIds() const { return m_vertexIds; }
    uint16_t* getVertexIdsPtr() { return m_vertexIds.data(); }
    
    // Get vertex ID for a specific corner of a specific side
    uint16_t getVertexId(int sideIndex, int cornerIndex) const {
        uint8_t vertexIndex = SIDE_VERTEX_TABLE[sideIndex][cornerIndex];
        return m_vertexIds[vertexIndex];
    }
    
    bool hasVertex(uint16_t vertexId) const;
    int findVertexIndex(uint16_t vertexId) const;
    bool updateVertexId(uint16_t oldId, uint16_t newId);
    
    // Sides
    Side& getSide(int index) { return m_sides[index]; }
    const Side& getSide(int index) const { return m_sides[index]; }
    std::array<Side, NUM_SIDES>& getSides() { return m_sides; }
    const std::array<Side, NUM_SIDES>& getSides() const { return m_sides; }
    
    // Children (connected segments)
    int16_t getChildId(int sideIndex) const { return m_sides[sideIndex].getChild(); }
    void setChildId(int sideIndex, int16_t childSegmentId);
    bool hasChild(int sideIndex) const { return m_sides[sideIndex].hasChild(); }
    bool replaceChild(int16_t oldChildId, int16_t newChildId);
    bool removeChild(int16_t childId) { return replaceChild(childId, -1); }
    void updateChildren(int16_t oldChildId, int16_t newChildId);
    
    // Function and properties
    SegmentFunction getFunction() const { return m_function; }
    void setFunction(SegmentFunction func) { m_function = func; }
    
    uint8_t getProperties() const { return m_properties; }
    void setProperties(uint8_t props) { m_properties = props; }
    bool hasProperty(SegmentProperty prop) const {
        return (m_properties & static_cast<uint8_t>(prop)) != 0;
    }
    void addProperty(SegmentProperty prop) {
        m_properties |= static_cast<uint8_t>(prop);
    }
    void removeProperty(SegmentProperty prop) {
        m_properties &= ~static_cast<uint8_t>(prop);
    }
    
    // Lighting
    int getStaticLight() const { return m_staticLight; }
    void setStaticLight(int light) { m_staticLight = light; }
    
    // Matcen/Producer
    int16_t getProducerId() const { return m_producerId; }
    void setProducerId(int16_t id) { m_producerId = id; }
    
    int16_t getValue() const { return m_value; }
    void setValue(int16_t value) { m_value = value; }
    
    // Damage (for lava, etc.)
    int16_t getDamage(int index) const { return m_damage[index]; }
    void setDamage(int index, int16_t damage) { m_damage[index] = damage; }
    
    // Center point
    const DoubleVector& getCenter() const { return m_center; }
    void setCenter(const DoubleVector& center) { m_center = center; }
    DoubleVector computeCenter() const;
    
    // Tagging (for selection, operations, etc.)
    enum TagMask : uint8_t {
        TAGGED = 0x01,
        SELECTED = 0x02,
        MARKED = 0x04
    };
    
    void tag(uint8_t mask = TAGGED) { m_tag |= mask; }
    void untag(uint8_t mask = TAGGED) { m_tag &= ~mask; }
    void toggleTag(uint8_t mask = TAGGED) { m_tag ^= mask; }
    bool isTagged(uint8_t mask = TAGGED) const { return (m_tag & mask) != 0; }
    
    // Tag sides
    void tagSide(int sideIndex, uint8_t mask = TAGGED) { m_sides[sideIndex].tag(mask); }
    void untagSide(int sideIndex, uint8_t mask = TAGGED) { m_sides[sideIndex].untag(mask); }
    void toggleTagSide(int sideIndex, uint8_t mask = TAGGED) { m_sides[sideIndex].toggleTag(mask); }
    bool isSideTagged(int sideIndex, uint8_t mask = TAGGED) const { return m_sides[sideIndex].isTagged(mask); }
    
    // Misc flags
    uint8_t getChildFlags() const { return m_childFlags; }
    void setChildFlags(uint8_t flags) { m_childFlags = flags; }
    
    uint8_t getWallFlags() const { return m_wallFlags; }
    void setWallFlags(uint8_t flags) { m_wallFlags = flags; }
    
    bool isTunnel() const { return m_tunnel; }
    void setTunnel(bool tunnel) { m_tunnel = tunnel; }
    
    int8_t getOwner() const { return m_owner; }
    void setOwner(int8_t owner) { m_owner = owner; }
    
    int8_t getGroup() const { return m_group; }
    void setGroup(int8_t group) { m_group = group; }
    
    // D2X detection
    bool isD2X() const {
        return static_cast<uint8_t>(m_function) >= static_cast<uint8_t>(SegmentFunction::TEAM_BLUE) 
            || m_properties != 0;
    }
    
    // Common operations
    int findCommonVertices(const Segment& other, int maxVertices, uint16_t* outVertices) const;
    int findCommonSide(int sideIndex, const Segment& other, int& outOtherSideIndex) const;
    
    // File I/O
    void read(QDataStream& stream);
    void write(QDataStream& stream) const;
    void readExtras(QDataStream& stream, bool hasExtras);
    void writeExtras(QDataStream& stream, bool hasExtras) const;

private:
    std::array<uint16_t, NUM_VERTICES_PER_SEGMENT> m_vertexIds;  // 8 vertex IDs
    std::array<Side, NUM_SIDES> m_sides;                         // 6 sides
    
    // Segment properties
    int m_staticLight;                // Average static light (0-0x7fff)
    SegmentFunction m_function;       // Primary function (matcen, goal, etc.)
    uint8_t m_properties;             // Property flags (water, lava, etc.)
    uint8_t m_tag;                    // Selection/marking flags
    uint8_t m_childFlags;             // Child connection flags (one bit per side)
    uint8_t m_wallFlags;              // Wall flags (one bit per side)
    
    // Producer/matcen
    int16_t m_producerId;             // Matcen ID
    int16_t m_value;                  // Producer value/index
    
    // Damage
    std::array<int16_t, 2> m_damage;  // Damage values [0]=shields, [1]=energy
    
    // Misc
    bool m_tunnel;                    // Is this part of a tunnel?
    int8_t m_owner;                   // Owner (multiplayer)
    int8_t m_group;                   // Group ID
    
    // Geometry cache
    DoubleVector m_center;            // Center point
};

} // namespace dle


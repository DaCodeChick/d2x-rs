#ifndef DLE_SIDE_H
#define DLE_SIDE_H

#include "../types/Types.h"
#include <memory>
#include <array>

namespace dle {

// Forward declarations
class Segment;
class Wall;
class Trigger;

// Texture masks and alignment
constexpr uint16_t TEXTURE_MASK = 0x3fff;
constexpr uint16_t ALIGNMENT_MASK = 0xc000;
constexpr int ALIGNMENT_SHIFT = 14;

// Default UVLs for a standard cube face
constexpr UVLS DEFAULT_UVLS[4] = {
    UVLS(0, 0, 0),
    UVLS(intToFix(1), 0, 0),
    UVLS(intToFix(1), intToFix(1), 0),
    UVLS(0, intToFix(1), 0)
};

/**
 * @brief Side represents one face of a segment (cube)
 * 
 * Each segment has 6 sides. Each side has:
 * - 4 vertices with UV coordinates and lighting
 * - Optional textures (base + overlay)
 * - Optional wall (door, destructible, etc.)
 * - Connection to a child segment (or -1 if solid)
 */
class Side {
public:
    Side();
    ~Side() = default;
    
    // Copy/move constructors
    Side(const Side&) = default;
    Side& operator=(const Side&) = default;
    Side(Side&&) = default;
    Side& operator=(Side&&) = default;
    
    // Initialization
    void clear();
    void reset(int sideIndex);
    
    // Parent segment
    void setParent(int16_t parentSegmentId) { m_parentSegmentId = parentSegmentId; }
    int16_t getParent() const { return m_parentSegmentId; }
    
    // Child segment (connected segment on the other side)
    void setChild(int16_t childSegmentId) { m_childSegmentId = childSegmentId; }
    int16_t getChild() const { return m_childSegmentId; }
    bool hasChild() const { return m_childSegmentId != -1; }
    
    // Wall
    void setWall(uint16_t wallId) { m_wallId = wallId; }
    uint16_t getWallId() const { return m_wallId; }
    bool hasWall() const { return m_wallId != 0xFFFF; }
    void deleteWall() { m_wallId = 0xFFFF; }
    
    // Textures
    void setBaseTexture(int16_t textureId) { m_baseTexture = textureId; }
    int16_t getBaseTexture() const { return m_baseTexture; }
    
    void setOverlayTexture(int16_t textureId) { m_overlayTexture = textureId; }
    int16_t getOverlayTexture(bool withAlignment = true) const {
        return withAlignment ? m_overlayTexture : (m_overlayTexture & TEXTURE_MASK);
    }
    int16_t getOverlayAlignment() const {
        return static_cast<int16_t>((m_overlayTexture & ALIGNMENT_MASK) >> ALIGNMENT_SHIFT);
    }
    
    bool setTextures(int16_t baseTexture, int16_t overlayTexture);
    void getTextures(int16_t& baseTexture, int16_t& overlayTexture) const;
    void resetTextures();
    
    // UV coordinates and lighting (4 vertices per side)
    UVLS& getUVL(int index) { return m_uvls[index]; }
    const UVLS& getUVL(int index) const { return m_uvls[index]; }
    const std::array<UVLS, 4>& getUVLs() const { return m_uvls; }
    void setUVL(int index, const UVLS& uvl) { m_uvls[index] = uvl; }
    void initUVL(int16_t textureId);
    
    // UV deltas (for animated textures or flickering lights)
    UVLS& getUVLDelta(int index) { return m_uvlDeltas[index]; }
    const UVLS& getUVLDelta(int index) const { return m_uvlDeltas[index]; }
    void setUVLDelta(int index, const UVLS& delta) { m_uvlDeltas[index] = delta; }
    
    // Vertex indices (into parent segment's vertex array)
    void setVertexIdIndex(int index, uint8_t vertexIdIndex) { m_vertexIdIndices[index] = vertexIdIndex; }
    uint8_t getVertexIdIndex(int index) const { return m_vertexIdIndices[index]; }
    const std::array<uint8_t, 4>& getVertexIdIndices() const { return m_vertexIdIndices; }
    
    // Shape (quad, triangle, etc.)
    enum class Shape : uint8_t {
        QUAD = 0,      // 4 vertices
        TRIANGLE_1 = 1, // 3 vertices (omit vertex 3)
        TRIANGLE_2 = 2, // 3 vertices (omit vertex 2)
        TRIANGLE_3 = 3  // 3 vertices (omit vertex 1)
    };
    
    void setShape(Shape shape) { m_shape = shape; }
    Shape getShape() const { return m_shape; }
    int getVertexCount() const;
    int getFaceCount() const;
    Shape detectShape();
    
    // Normal vector
    const DoubleVector& getNormal() const { return m_normal; }
    void setNormal(const DoubleVector& normal) { m_normal = normal; }
    DoubleVector computeNormal(const std::array<uint16_t, 8>& vertexIds, const DoubleVector& center);
    
    // Center point
    const DoubleVector& getCenter() const { return m_center; }
    void setCenter(const DoubleVector& center) { m_center = center; }
    
    // Visibility
    bool isVisible() const;
    
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
    
    // File I/O
    void read(class FileReader& reader, bool textured);
    void write(class FileWriter& writer, bool textured) const;

private:
    int16_t m_parentSegmentId;           // Parent segment index
    int16_t m_childSegmentId;            // Child segment index (-1 if solid wall)
    uint16_t m_wallId;                   // Wall index (0xFFFF if no wall)
    int16_t m_baseTexture;               // Base texture index
    int16_t m_overlayTexture;            // Overlay texture + alignment bits
    std::array<UVLS, 4> m_uvls;          // UV coords + light for 4 vertices
    std::array<UVLS, 4> m_uvlDeltas;     // Deltas for animation
    std::array<uint8_t, 4> m_vertexIdIndices; // Indices into parent segment's vertex ID array
    Shape m_shape;                       // Quad or triangle shape
    uint8_t m_tag;                       // Selection/marking flags
    DoubleVector m_normal;               // Face normal vector
    DoubleVector m_center;               // Face center point
    
    static constexpr int VERTEX_COUNTS[4] = {4, 3, 3, 3};
    static constexpr int FACE_COUNTS[4] = {2, 1, 1, 1};
};

} // namespace dle

#endif // DLE_SIDE_H

#include "Side.h"
#include <QDataStream>
#include <cstring>

namespace dle {

// Helper functions for reading/writing Descent types
namespace {
    UVLS readUVLS(QDataStream& stream) {
        int32_t u, v;
        uint16_t light;
        stream >> u >> v >> light;
        return UVLS(u, v, light);
    }
    
    void writeUVLS(QDataStream& stream, const UVLS& uvls) {
        stream << uvls.u << uvls.v << uvls.light;
    }
}

Side::Side()
    : m_parentSegmentId(-1)
    , m_childSegmentId(-1)
    , m_wallId(0xFFFF)
    , m_baseTexture(0)
    , m_overlayTexture(0)
    , m_shape(Shape::QUAD)
    , m_tag(0)
{
    for (int i = 0; i < 4; ++i) {
        m_uvls[i] = DEFAULT_UVLS[i];
        m_uvlDeltas[i] = UVLS();
        m_vertexIdIndices[i] = 0;
    }
}

void Side::clear() {
    m_parentSegmentId = -1;
    m_childSegmentId = -1;
    m_wallId = 0xFFFF;
    m_baseTexture = 0;
    m_overlayTexture = 0;
    m_shape = Shape::QUAD;
    m_tag = 0;
    
    for (int i = 0; i < 4; ++i) {
        m_uvls[i] = UVLS();
        m_uvlDeltas[i] = UVLS();
        m_vertexIdIndices[i] = 0;
    }
    
    m_normal = DoubleVector(0, 0, 0);
    m_center = DoubleVector(0, 0, 0);
}

void Side::reset(int sideIndex) {
    clear();
    
    // Initialize with default UVLs
    for (int i = 0; i < 4; ++i) {
        m_uvls[i] = DEFAULT_UVLS[i];
    }
}

bool Side::setTextures(int16_t baseTexture, int16_t overlayTexture) {
    m_baseTexture = baseTexture;
    m_overlayTexture = overlayTexture;
    return true;
}

void Side::getTextures(int16_t& baseTexture, int16_t& overlayTexture) const {
    baseTexture = m_baseTexture;
    overlayTexture = m_overlayTexture;
}

void Side::resetTextures() {
    m_baseTexture = 0;
    m_overlayTexture = 0;
}

void Side::initUVL(int16_t textureId) {
    // Initialize UV coordinates to default square mapping
    for (int i = 0; i < 4; ++i) {
        m_uvls[i] = DEFAULT_UVLS[i];
    }
}

int Side::getVertexCount() const {
    return VERTEX_COUNTS[static_cast<int>(m_shape)];
}

int Side::getFaceCount() const {
    return FACE_COUNTS[static_cast<int>(m_shape)];
}

Side::Shape Side::detectShape() {
    // TODO: Detect shape based on vertex positions
    // For now, assume all sides are quads
    return Shape::QUAD;
}

DoubleVector Side::computeNormal(const std::array<uint16_t, 8>& vertexIds, const DoubleVector& center) {
    // TODO: Compute normal from vertex positions
    // This requires access to the vertex array which we'll implement later
    return DoubleVector(0, 0, 1);
}

bool Side::isVisible() const {
    // A side is visible if it doesn't have a child segment (it's not connected to another segment)
    // or if it has a wall (door, grate, etc.)
    return !hasChild() || hasWall();
}

void Side::read(QDataStream& stream, bool textured) {
    // Read side data from file
    uint8_t wallNum;
    stream >> wallNum;
    m_wallId = wallNum;  // Wall number (255 = no wall)
    stream >> m_baseTexture;
    
    if (textured) {
        stream >> m_overlayTexture;
        
        // Read UVLs (4 per side)
        for (int i = 0; i < 4; ++i) {
            m_uvls[i] = readUVLS(stream);
        }
    } else {
        m_overlayTexture = 0;
        for (int i = 0; i < 4; ++i) {
            m_uvls[i] = DEFAULT_UVLS[i];
        }
    }
}

void Side::write(QDataStream& stream, bool textured) const {
    // Write side data to file
    uint8_t wallNum = (m_wallId == 0xFFFF ? 255 : static_cast<uint8_t>(m_wallId));
    stream << wallNum << m_baseTexture;
    
    if (textured) {
        stream << m_overlayTexture;
        
        // Write UVLs (4 per side)
        for (int i = 0; i < 4; ++i) {
            writeUVLS(stream, m_uvls[i]);
        }
    }
}

} // namespace dle

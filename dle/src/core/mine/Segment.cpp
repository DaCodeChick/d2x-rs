#include "Segment.h"
#include <QDataStream>
#include <algorithm>
#include <cstring>

namespace dle {

Segment::Segment()
    : m_staticLight(0)
    , m_function(SegmentFunction::NONE)
    , m_properties(0)
    , m_tag(0)
    , m_childFlags(0)
    , m_wallFlags(0)
    , m_producerId(-1)
    , m_value(0)
    , m_tunnel(false)
    , m_owner(-1)
    , m_group(-1)
{
    m_vertexIds.fill(0);
    m_damage.fill(0);
    
    // Initialize all sides
    for (int i = 0; i < NUM_SIDES; ++i) {
        m_sides[i].clear();
    }
}

void Segment::clear() {
    m_vertexIds.fill(0);
    m_staticLight = 0;
    m_function = SegmentFunction::NONE;
    m_properties = 0;
    m_tag = 0;
    m_childFlags = 0;
    m_wallFlags = 0;
    m_producerId = -1;
    m_value = 0;
    m_damage.fill(0);
    m_tunnel = false;
    m_owner = -1;
    m_group = -1;
    m_center = DoubleVector(0, 0, 0);
    
    for (int i = 0; i < NUM_SIDES; ++i) {
        m_sides[i].clear();
    }
}

void Segment::setup() {
    // Initialize side vertex indices based on the standard cube layout
    for (int sideIndex = 0; sideIndex < NUM_SIDES; ++sideIndex) {
        for (int cornerIndex = 0; cornerIndex < NUM_VERTICES_PER_SIDE; ++cornerIndex) {
            uint8_t vertexIndex = SIDE_VERTEX_TABLE[sideIndex][cornerIndex];
            m_sides[sideIndex].setVertexIdIndex(cornerIndex, vertexIndex);
        }
    }
}

void Segment::reset(int sideIndex) {
    if (sideIndex >= 0 && sideIndex < NUM_SIDES) {
        m_sides[sideIndex].reset(sideIndex);
    } else {
        // Reset all sides
        for (int i = 0; i < NUM_SIDES; ++i) {
            m_sides[i].reset(i);
        }
    }
}

bool Segment::hasVertex(uint16_t vertexId) const {
    return std::find(m_vertexIds.begin(), m_vertexIds.end(), vertexId) != m_vertexIds.end();
}

int Segment::findVertexIndex(uint16_t vertexId) const {
    auto it = std::find(m_vertexIds.begin(), m_vertexIds.end(), vertexId);
    if (it != m_vertexIds.end()) {
        return static_cast<int>(std::distance(m_vertexIds.begin(), it));
    }
    return -1;
}

bool Segment::updateVertexId(uint16_t oldId, uint16_t newId) {
    bool updated = false;
    for (auto& vertexId : m_vertexIds) {
        if (vertexId == oldId) {
            vertexId = newId;
            updated = true;
        }
    }
    return updated;
}

void Segment::setChildId(int sideIndex, int16_t childSegmentId) {
    m_sides[sideIndex].setChild(childSegmentId);
    
    // Update child flags
    if (childSegmentId >= 0) {
        m_childFlags |= (1 << sideIndex);
    } else {
        m_childFlags &= ~(1 << sideIndex);
    }
}

bool Segment::replaceChild(int16_t oldChildId, int16_t newChildId) {
    bool replaced = false;
    for (int i = 0; i < NUM_SIDES; ++i) {
        if (m_sides[i].getChild() == oldChildId) {
            setChildId(i, newChildId);
            replaced = true;
        }
    }
    return replaced;
}

void Segment::updateChildren(int16_t oldChildId, int16_t newChildId) {
    replaceChild(oldChildId, newChildId);
}

DoubleVector Segment::computeCenter() const {
    // TODO: Compute center from vertex positions
    // This requires access to the vertex array which we'll implement later
    return DoubleVector(0, 0, 0);
}

int Segment::findCommonVertices(const Segment& other, int maxVertices, uint16_t* outVertices) const {
    int count = 0;
    for (uint16_t myVertex : m_vertexIds) {
        if (count >= maxVertices) break;
        if (other.hasVertex(myVertex)) {
            if (outVertices) {
                outVertices[count] = myVertex;
            }
            ++count;
        }
    }
    return count;
}

int Segment::findCommonSide(int sideIndex, const Segment& other, int& outOtherSideIndex) const {
    // Find common vertices for this side
    uint16_t sideVertices[4];
    for (int i = 0; i < NUM_VERTICES_PER_SIDE; ++i) {
        sideVertices[i] = getVertexId(sideIndex, i);
    }
    
    // Check each side of the other segment
    for (int otherSide = 0; otherSide < NUM_SIDES; ++otherSide) {
        int commonCount = 0;
        for (int i = 0; i < NUM_VERTICES_PER_SIDE; ++i) {
            uint16_t otherVertex = other.getVertexId(otherSide, i);
            for (int j = 0; j < NUM_VERTICES_PER_SIDE; ++j) {
                if (sideVertices[j] == otherVertex) {
                    ++commonCount;
                    break;
                }
            }
        }
        
        // If all 4 vertices match, we found the common side
        if (commonCount == NUM_VERTICES_PER_SIDE) {
            outOtherSideIndex = otherSide;
            return commonCount;
        }
    }
    
    return 0;
}

void Segment::read(QDataStream& stream) {
    // Read segment data structure (basic D1/D2 format)
    // Read sides first (6 sides)
    for (int i = 0; i < NUM_SIDES; ++i) {
        m_sides[i].read(stream, true);
    }
    
    // Read children (6 child indices)
    for (int i = 0; i < NUM_SIDES; ++i) {
        int16_t childId;
        stream >> childId;
        setChildId(i, childId);
    }
    
    // Read vertex IDs (8 vertices)
    for (int i = 0; i < NUM_VERTICES_PER_SEGMENT; ++i) {
        stream >> m_vertexIds[i];
    }
    
    // Read special attributes
    stream >> m_staticLight;  // Static light value
    
    // Skip wall bitmap mask (not used in modern editor)
    stream.skipRawData(2);
}

void Segment::write(QDataStream& stream) const {
    // Write segment data structure (basic D1/D2 format)
    // Write sides first (6 sides)
    for (int i = 0; i < NUM_SIDES; ++i) {
        m_sides[i].write(stream, true);
    }
    
    // Write children (6 child indices)
    for (int i = 0; i < NUM_SIDES; ++i) {
        stream << m_sides[i].getChild();
    }
    
    // Write vertex IDs (8 vertices)
    for (int i = 0; i < NUM_VERTICES_PER_SEGMENT; ++i) {
        stream << m_vertexIds[i];
    }
    
    // Write special attributes
    stream << m_staticLight;
    
    // Write wall bitmap mask (set to 0)
    stream << static_cast<uint16_t>(0);
}

void Segment::readExtras(QDataStream& stream, bool hasExtras) {
    if (!hasExtras) {
        return;
    }
    
    // Read D2X-XL extended attributes
    uint8_t specialType, props;
    int16_t val, s2flags;
    int32_t lightColor, fadeTime;
    stream >> specialType >> props >> val >> s2flags >> lightColor >> fadeTime;
}

void Segment::writeExtras(QDataStream& stream, bool hasExtras) const {
    if (!hasExtras) {
        return;
    }
    
    // Write D2X-XL extended attributes  
    stream << static_cast<uint8_t>(m_function)
           << m_properties
           << m_value
           << static_cast<int16_t>(0)  // S2 flags (unused)
           << static_cast<int32_t>(0)  // Light color (RGBA) - default
           << static_cast<int32_t>(0); // Fade time - default
}

} // namespace dle

#include "Segment.h"
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

void Segment::read(class FileReader& reader) {
    // TODO: Implement file reading
    // This will be implemented when we create the FileReader class
}

void Segment::write(class FileWriter& writer) const {
    // TODO: Implement file writing
    // This will be implemented when we create the FileWriter class
}

void Segment::readExtras(class FileReader& reader, bool hasExtras) {
    // TODO: Implement extras reading
    // This will be implemented when we create the FileReader class
}

void Segment::writeExtras(class FileWriter& writer, bool hasExtras) const {
    // TODO: Implement extras writing
    // This will be implemented when we create the FileWriter class
}

} // namespace dle

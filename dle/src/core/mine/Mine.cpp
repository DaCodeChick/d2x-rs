#include "Mine.h"
#include <algorithm>

namespace dle {

Mine::Mine()
    : m_fileType(FileType::RL2)
    , m_levelVersion(LEVEL_VERSION_D2)
    , m_reactorTime(45)
    , m_reactorStrength(-1)
    , m_secretSegment(-1)
    , m_changesMade(false)
{
}

void Mine::clear() {
    m_vertices.clear();
    m_segments.clear();
    m_levelName.clear();
    m_hostageText.clear();
    m_reactorTime = 45;
    m_reactorStrength = -1;
    m_secretSegment = -1;
    m_secretOrientation = Matrix();
    m_changesMade = false;
}

void Mine::initialize() {
    clear();
}

void Mine::reset() {
    clear();
    createDefault();
}

void Mine::removeVertex(int index) {
    if (index < 0 || index >= getVertexCount()) {
        return;
    }
    
    // TODO: Update all segments that reference this vertex
    // For now, just remove it
    m_vertices.erase(m_vertices.begin() + index);
    
    // Update all vertex IDs that are greater than the removed index
    for (auto& segment : m_segments) {
        for (int i = 0; i < NUM_VERTICES_PER_SEGMENT; ++i) {
            uint16_t vertexId = segment.getVertexId(i);
            if (vertexId > static_cast<uint16_t>(index)) {
                segment.setVertexId(i, vertexId - 1);
            }
        }
    }
    
    markChanged();
}

void Mine::removeSegment(int index) {
    if (index < 0 || index >= getSegmentCount()) {
        return;
    }
    
    // Remove connections from other segments to this one
    for (auto& segment : m_segments) {
        for (int side = 0; side < NUM_SIDES; ++side) {
            if (segment.getChildId(side) == index) {
                segment.setChildId(side, -1);
            }
        }
    }
    
    m_segments.erase(m_segments.begin() + index);
    
    // Update all segment IDs that are greater than the removed index
    for (auto& segment : m_segments) {
        for (int side = 0; side < NUM_SIDES; ++side) {
            int16_t childId = segment.getChildId(side);
            if (childId > index) {
                segment.setChildId(side, childId - 1);
            }
        }
    }
    
    markChanged();
}

void Mine::createDefault() {
    clear();
    
    // Create 8 vertices for a single cube centered at origin
    // Standard cube with 20x20x20 size (in fixed point)
    const fix cubeSize = intToFix(20);
    const fix halfSize = cubeSize / 2;
    
    // Cube vertices (standard Descent order)
    addVertex(-halfSize, -halfSize, -halfSize); // 0: Front bottom left
    addVertex( halfSize, -halfSize, -halfSize); // 1: Front bottom right
    addVertex( halfSize,  halfSize, -halfSize); // 2: Front top right
    addVertex(-halfSize,  halfSize, -halfSize); // 3: Front top left
    addVertex(-halfSize, -halfSize,  halfSize); // 4: Back bottom left
    addVertex( halfSize, -halfSize,  halfSize); // 5: Back bottom right
    addVertex( halfSize,  halfSize,  halfSize); // 6: Back top right
    addVertex(-halfSize,  halfSize,  halfSize); // 7: Back top left
    
    // Create one segment
    Segment segment;
    segment.clear();
    segment.setup();
    
    // Set vertex IDs
    for (int i = 0; i < NUM_VERTICES_PER_SEGMENT; ++i) {
        segment.setVertexId(i, i);
    }
    
    // Initialize all sides
    for (int i = 0; i < NUM_SIDES; ++i) {
        segment.getSide(i).setChild(-1);  // No connections
        segment.getSide(i).setBaseTexture(0);
        segment.getSide(i).setOverlayTexture(0);
        segment.getSide(i).initUVL(0);
    }
    
    addSegment(segment);
    
    m_levelName = "Untitled";
    m_changesMade = false;
}

bool Mine::load(const std::string& filename) {
    // TODO: Implement file loading
    // This will be implemented when we create the file I/O classes
    return false;
}

bool Mine::save(const std::string& filename) {
    // TODO: Implement file saving
    // This will be implemented when we create the file I/O classes
    return false;
}

} // namespace dle

#include "Mine.h"
#include "../io/FileReader.h"
#include "../io/FileWriter.h"
#include <QString>
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
    FileReader reader;
    if (!reader.open(QString::fromStdString(filename))) {
        return false;
    }
    
    // Detect file type from extension
    QString qfilename = QString::fromStdString(filename);
    if (qfilename.endsWith(".rdl", Qt::CaseInsensitive)) {
        m_fileType = FileType::RDL;
        m_levelVersion = LEVEL_VERSION_D1;
    } else if (qfilename.endsWith(".rl2", Qt::CaseInsensitive)) {
        m_fileType = FileType::RL2;
        m_levelVersion = LEVEL_VERSION_D2;
    } else {
        return false;  // Unknown file type
    }
    
    // Read header
    uint8_t compiledVersion = reader.readUInt8();
    if (compiledVersion != 0) {
        return false;  // Invalid file
    }
    
    // Read counts (new file format uses u16)
    uint16_t vertexCount = reader.readUInt16();
    uint16_t segmentCount = reader.readUInt16();
    
    // Read vertices
    m_vertices.clear();
    m_vertices.reserve(vertexCount);
    for (int i = 0; i < vertexCount; ++i) {
        Vector pos = reader.readVector();
        m_vertices.push_back(Vertex(pos));
    }
    
    // Read segments
    m_segments.clear();
    m_segments.reserve(segmentCount);
    for (int i = 0; i < segmentCount; ++i) {
        Segment segment;
        segment.read(reader);
        m_segments.push_back(segment);
    }
    
    // TODO: Read walls, triggers, objects, etc.
    
    m_levelName = QString::fromStdString(filename).section('/', -1).section('.', 0, 0).toStdString();
    m_changesMade = false;
    
    return !reader.hasError();
}

bool Mine::save(const std::string& filename) {
    FileWriter writer;
    if (!writer.open(QString::fromStdString(filename))) {
        return false;
    }
    
    // Write header
    writer.writeUInt8(0);  // Compiled version
    writer.writeUInt16(static_cast<uint16_t>(m_vertices.size()));
    writer.writeUInt16(static_cast<uint16_t>(m_segments.size()));
    
    // Write vertices
    for (const auto& vertex : m_vertices) {
        writer.writeVector(vertex.position);
    }
    
    // Write segments
    for (const auto& segment : m_segments) {
        segment.write(writer);
    }
    
    // TODO: Write walls, triggers, objects, etc.
    
    writer.flush();
    m_changesMade = false;
    
    return !writer.hasError();
}

} // namespace dle

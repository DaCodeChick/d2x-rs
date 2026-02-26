#include "Mine.h"
#include "../io/FileReader.h"
#include "../io/FileWriter.h"
#include <QString>
#include <QDebug>
#include <QtTypes>
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
    m_walls.clear();
    m_triggers.clear();
    m_objects.clear();
    m_robotMakers.clear();
    m_equipmentMakers.clear();
    m_reactorTrigger.reset();
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
    
    // Read LVLP signature
    char fileSignature[4];
    reader.readBytes(fileSignature, 4);
    if (fileSignature[0] != 'L' || fileSignature[1] != 'V' || fileSignature[2] != 'L' || fileSignature[3] != 'P') {
        return false;
    }
    
    // Read file version
    int32_t fileVersion = reader.readInt32();
    
    // Determine file type based on version
    if (fileVersion == 1) {
        m_fileType = FileType::RDL;
        m_levelVersion = LEVEL_VERSION_D1;
    } else if (fileVersion >= 6 && fileVersion <= LEVEL_VERSION_D2) {
        m_fileType = FileType::RL2;
        m_levelVersion = fileVersion;
    } else {
        return false;
    }
    
    // Read mine and game data offsets
    int32_t mineDataOffset = reader.readInt32();
    int32_t gameDataOffset = reader.readInt32();
    
    // Skip palette name and D2-specific reactor info for now
    // (We'll implement these later)
    
    // Seek to mine data and load geometry
    reader.seek(mineDataOffset);
    
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
    
    // Seek to game data section
    reader.seek(gameDataOffset);
    
    // Read game data signature (0x6705)
    uint16_t signature = reader.readUInt16();
    if (signature != 0x6705) {
        return false;  // Invalid game data signature
    }
    
    // Read game data version (different from LVLP file version)
    int32_t gameVersion = reader.readInt32();
    // Note: gameVersion is save format version, not the same as m_levelVersion (D1/D2 distinction)
    
    // Read level name (if version >= 14)
    if (gameVersion >= 14) {
        char nameBuffer[256] = {0};
        int nameIdx = 0;
        while (nameIdx < 255) {
            char ch = reader.readInt8();
            if (ch == '\n') ch = 0;
            nameBuffer[nameIdx++] = ch;
            if (ch == 0) break;
        }
        m_levelName = nameBuffer;
    }
    
    // Read objects
    int32_t objectCount = reader.readInt32();
    m_objects.clear();
    m_objects.reserve(std::min(objectCount, getMaxObjects()));
    for (int i = 0; i < objectCount && i < getMaxObjects(); ++i) {
        Object obj;
        obj.read(reader, m_levelVersion);
        m_objects.push_back(obj);
    }
    
    // Read walls
    int32_t wallCount = reader.readInt32();
    m_walls.clear();
    m_walls.reserve(std::min(wallCount, getMaxWalls()));
    for (int i = 0; i < wallCount && i < getMaxWalls(); ++i) {
        Wall wall;
        wall.read(reader, m_levelVersion);
        m_walls.push_back(wall);
    }
    
    // Read triggers (wall triggers)
    int32_t triggerCount = reader.readInt32();
    m_triggers.clear();
    m_triggers.reserve(std::min(triggerCount, getMaxTriggers()));
    for (int i = 0; i < triggerCount && i < getMaxTriggers(); ++i) {
        Trigger trigger;
        trigger.read(reader, false, m_levelVersion);  // false = wall trigger
        m_triggers.push_back(trigger);
    }
    
    // Read reactor trigger (if present)
    int32_t reactorTriggerCount = reader.readInt32();
    if (reactorTriggerCount > 0) {
        ReactorTrigger reactorTrigger;
        reactorTrigger.read(reader);
        m_reactorTrigger = reactorTrigger;
    }
    
    // Read robot makers (matcens)
    int32_t robotMakerCount = reader.readInt32();
    m_robotMakers.clear();
    m_robotMakers.reserve(std::min(robotMakerCount, getMaxMatcens()));
    for (int i = 0; i < robotMakerCount && i < getMaxMatcens(); ++i) {
        Matcen matcen;
        matcen.read(reader, m_levelVersion);
        m_robotMakers.push_back(matcen);
    }
    
    // Read equipment makers (D2 only)
    if (m_levelVersion >= LEVEL_VERSION_D2) {
        int32_t equipmentMakerCount = reader.readInt32();
        m_equipmentMakers.clear();
        m_equipmentMakers.reserve(std::min(equipmentMakerCount, getMaxMatcens()));
        for (int i = 0; i < equipmentMakerCount && i < getMaxMatcens(); ++i) {
            Matcen matcen;
            matcen.read(reader, m_levelVersion);
            m_equipmentMakers.push_back(matcen);
        }
        
        // TODO: Read light deltas (D2 only)
    }
    
    if (m_levelName.empty()) {
        m_levelName = QString::fromStdString(filename).section('/', -1).section('.', 0, 0).toStdString();
    }
    
    m_changesMade = false;
    
    return !reader.hasError();
}

bool Mine::save(const std::string& filename) {
    FileWriter writer;
    if (!writer.open(QString::fromStdString(filename))) {
        return false;
    }
    
    // Write LVLP signature
    writer.writeInt8('L');
    writer.writeInt8('V');
    writer.writeInt8('L');
    writer.writeInt8('P');
    
    // Write file version
    int32_t fileVersion = (m_fileType == FileType::RDL) ? 1 : m_levelVersion;
    writer.writeInt32(fileVersion);
    
    // Reserve space for offsets (we'll come back and write these)
    qint64 mineOffsetPos = writer.pos();
    writer.writeInt32(0);  // Placeholder for mine data offset
    writer.writeInt32(0);  // Placeholder for game data offset
    
    // For D1 files, write 4 padding bytes (observed in original files)
    if (m_fileType == FileType::RDL) {
        writer.writeInt32(0);
    }
    
    // Skip D2-specific data for now (palette, reactor, secret segment)
    // In a full implementation, we'd write these here
    
    // Remember where mine data starts
    qint64 mineDataOffset = writer.pos();
    
    // Write mine geometry
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
    
    // Remember where game data starts
    qint64 gameDataOffset = writer.pos();
    
    // Write game data signature (0x6705)
    writer.writeUInt16(0x6705);
    
    // Write level version
    writer.writeInt32(m_levelVersion);
    
    // Write level name (if version >= 14)
    if (m_levelVersion >= 14) {
        const char* name = m_levelName.empty() ? "Untitled" : m_levelName.c_str();
        for (const char* p = name; *p != 0; ++p) {
            writer.writeInt8(*p);
        }
        writer.writeInt8(0);  // Null terminator
    }
    
    // Write objects
    writer.writeInt32(static_cast<int32_t>(m_objects.size()));
    for (const auto& obj : m_objects) {
        obj.write(writer, m_levelVersion);
    }
    
    // Write walls
    writer.writeInt32(static_cast<int32_t>(m_walls.size()));
    for (const auto& wall : m_walls) {
        wall.write(writer, m_levelVersion);
    }
    
    // Write triggers
    writer.writeInt32(static_cast<int32_t>(m_triggers.size()));
    for (const auto& trigger : m_triggers) {
        trigger.write(writer, false, m_levelVersion);  // false = wall trigger
    }
    
    // Write reactor trigger
    if (m_reactorTrigger.has_value()) {
        writer.writeInt32(1);  // Count = 1
        m_reactorTrigger->write(writer);
    } else {
        writer.writeInt32(0);  // Count = 0
    }
    
    // Write robot makers
    writer.writeInt32(static_cast<int32_t>(m_robotMakers.size()));
    for (const auto& matcen : m_robotMakers) {
        matcen.write(writer, m_levelVersion);
    }
    
    // Write equipment makers (D2 only)
    if (m_levelVersion >= LEVEL_VERSION_D2) {
        writer.writeInt32(static_cast<int32_t>(m_equipmentMakers.size()));
        for (const auto& matcen : m_equipmentMakers) {
            matcen.write(writer, m_levelVersion);
        }
        
        // TODO: Write light deltas (D2 only)
    }
    
    // Now go back and write the actual offsets
    qint64 endPos = writer.pos();
    writer.seek(mineOffsetPos);
    writer.writeInt32(static_cast<int32_t>(mineDataOffset));
    writer.writeInt32(static_cast<int32_t>(gameDataOffset));
    writer.seek(endPos);
    
    writer.flush();
    m_changesMade = false;
    
    return !writer.hasError();
}

} // namespace dle

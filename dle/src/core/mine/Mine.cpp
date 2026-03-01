#include "Mine.h"
#include "../io/LevelReader.h"
#include <QDataStream>
#include <QFile>
#include <print>

namespace dle {

// Helper functions for reading Descent binary format
namespace {

Vector readVector(QDataStream& stream) {
    int32_t x, y, z;
    stream >> x >> y >> z;
    return Vector(x, y, z);
}

Matrix readMatrix(QDataStream& stream) {
    Vector right = readVector(stream);
    Vector up = readVector(stream);
    Vector forward = readVector(stream);
    return Matrix(right, up, forward);
}

UVLS readUVLS(QDataStream& stream) {
    int32_t u, v;
    uint16_t light;
    stream >> u >> v >> light;
    return UVLS(u, v, light);
}

void writeVector(QDataStream& stream, const Vector& v) {
    stream << v.x << v.y << v.z;
}

void writeMatrix(QDataStream& stream, const Matrix& m) {
    writeVector(stream, m.right);
    writeVector(stream, m.up);
    writeVector(stream, m.forward);
}

void writeUVLS(QDataStream& stream, const UVLS& uvls) {
    stream << uvls.u << uvls.v << uvls.light;
}

} // anonymous namespace

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
    QFile file(QString::fromStdString(filename));
    if (!file.open(QIODevice::ReadOnly)) {
        return false;
    }
    
    // Peek at first 4 bytes to detect file format
    char fileSignature[4];
    if (file.read(fileSignature, 4) != 4) {
        return false;
    }
    file.seek(0);  // Reset to beginning
    
    // Check if this is an LVLP file (has "LVLP" signature)
    bool isLVLP = (fileSignature[0] == 'L' && fileSignature[1] == 'V' && 
                   fileSignature[2] == 'L' && fileSignature[3] == 'P');
    
    if (!isLVLP) {
        // This is a raw RDL/RL2 mine file - use LevelReader
        file.close();
        auto result = LevelReader::load(filename, *this);
        if (!result) {
            std::println(stderr, "Failed to load RDL/RL2 file: {}", result.error().message);
            return false;
        }
        m_changesMade = false;
        return true;
    }
    
    // LVLP format - continue with existing parser
    QDataStream stream(&file);
    stream.setByteOrder(QDataStream::LittleEndian);
    
    // Read LVLP signature (already validated above)
    stream.readRawData(fileSignature, 4);
    
    // Read file version
    int32_t fileVersion;
    stream >> fileVersion;
    
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
    int32_t mineDataOffset, gameDataOffset;
    stream >> mineDataOffset >> gameDataOffset;
    
    // Skip palette name and D2-specific reactor info for now
    // (We'll implement these later)
    
    // Seek to mine data and load geometry
    file.seek(mineDataOffset);
    
    // Read header
    uint8_t compiledVersion;
    stream >> compiledVersion;
    if (compiledVersion != 0) {
        return false;  // Invalid file
    }
    
    // Read counts (new file format uses u16)
    uint16_t vertexCount, segmentCount;
    stream >> vertexCount >> segmentCount;
    
    // Read vertices
    m_vertices.clear();
    m_vertices.reserve(vertexCount);
    for (int i = 0; i < vertexCount; ++i) {
        Vector pos = readVector(stream);
        m_vertices.push_back(Vertex(pos));
    }
    
    // Read segments
    m_segments.clear();
    m_segments.reserve(segmentCount);
    for (int i = 0; i < segmentCount; ++i) {
        Segment segment;
        segment.read(stream);
        m_segments.push_back(segment);
    }
    
    // Seek to game data section
    file.seek(gameDataOffset);
    
    // Read game data signature (0x6705)
    uint16_t signature;
    stream >> signature;
    if (signature != 0x6705) {
        return false;  // Invalid game data signature
    }
    
    // Read game data version (different from LVLP file version)
    int32_t gameVersion;
    stream >> gameVersion;
    // Note: gameVersion is save format version, not the same as m_levelVersion (D1/D2 distinction)
    
    // Read level name (if version >= 14)
    if (gameVersion >= 14) {
        char nameBuffer[256] = {0};
        int nameIdx = 0;
        while (nameIdx < 255) {
            int8_t ch;
            stream >> ch;
            if (ch == '\n') ch = 0;
            nameBuffer[nameIdx++] = ch;
            if (ch == 0) break;
        }
        m_levelName = nameBuffer;
    }
    
    // Read objects
    int32_t objectCount;
    stream >> objectCount;
    m_objects.clear();
    m_objects.reserve(std::min(objectCount, getMaxObjects()));
    for (int i = 0; i < objectCount && i < getMaxObjects(); ++i) {
        Object obj;
        obj.read(stream, m_levelVersion);
        m_objects.push_back(obj);
    }
    
    // Read walls
    int32_t wallCount;
    stream >> wallCount;
    m_walls.clear();
    m_walls.reserve(std::min(wallCount, getMaxWalls()));
    for (int i = 0; i < wallCount && i < getMaxWalls(); ++i) {
        Wall wall;
        wall.read(stream, m_levelVersion);
        m_walls.push_back(wall);
    }
    
    // Read triggers (wall triggers)
    int32_t triggerCount;
    stream >> triggerCount;
    m_triggers.clear();
    m_triggers.reserve(std::min(triggerCount, getMaxTriggers()));
    for (int i = 0; i < triggerCount && i < getMaxTriggers(); ++i) {
        Trigger trigger;
        trigger.read(stream, false, m_levelVersion);  // false = wall trigger
        m_triggers.push_back(trigger);
    }
    
    // Read reactor trigger (if present)
    int32_t reactorTriggerCount;
    stream >> reactorTriggerCount;
    if (reactorTriggerCount > 0) {
        ReactorTrigger reactorTrigger;
        reactorTrigger.read(stream);
        m_reactorTrigger = reactorTrigger;
    }
    
    // Read robot makers (matcens)
    int32_t robotMakerCount;
    stream >> robotMakerCount;
    m_robotMakers.clear();
    m_robotMakers.reserve(std::min(robotMakerCount, getMaxMatcens()));
    for (int i = 0; i < robotMakerCount && i < getMaxMatcens(); ++i) {
        Matcen matcen;
        matcen.read(stream, m_levelVersion);
        m_robotMakers.push_back(matcen);
    }
    
    // Read equipment makers (D2 only)
    if (m_levelVersion >= LEVEL_VERSION_D2) {
        int32_t equipmentMakerCount;
        stream >> equipmentMakerCount;
        m_equipmentMakers.clear();
        m_equipmentMakers.reserve(std::min(equipmentMakerCount, getMaxMatcens()));
        for (int i = 0; i < equipmentMakerCount && i < getMaxMatcens(); ++i) {
            Matcen matcen;
            matcen.read(stream, m_levelVersion);
            m_equipmentMakers.push_back(matcen);
        }
        
        // TODO: Read light deltas (D2 only)
    }
    
    if (m_levelName.empty()) {
        m_levelName = QString::fromStdString(filename).section('/', -1).section('.', 0, 0).toStdString();
    }
    
    m_changesMade = false;
    
    return stream.status() == QDataStream::Ok;
}

bool Mine::save(const std::string& filename) {
    QFile file(QString::fromStdString(filename));
    if (!file.open(QIODevice::WriteOnly)) {
        return false;
    }
    
    QDataStream stream(&file);
    stream.setByteOrder(QDataStream::LittleEndian);
    
    // Write LVLP signature
    stream << static_cast<int8_t>('L');
    stream << static_cast<int8_t>('V');
    stream << static_cast<int8_t>('L');
    stream << static_cast<int8_t>('P');
    
    // Write file version
    int32_t fileVersion = (m_fileType == FileType::RDL) ? 1 : m_levelVersion;
    stream << fileVersion;
    
    // Reserve space for offsets (we'll come back and write these)
    qint64 mineOffsetPos = file.pos();
    stream << static_cast<int32_t>(0);  // Placeholder for mine data offset
    stream << static_cast<int32_t>(0);  // Placeholder for game data offset
    
    // For D1 files, write 4 padding bytes (observed in original files)
    if (m_fileType == FileType::RDL) {
        stream << static_cast<int32_t>(0);
    }
    
    // Skip D2-specific data for now (palette, reactor, secret segment)
    // In a full implementation, we'd write these here
    
    // Remember where mine data starts
    qint64 mineDataOffset = file.pos();
    
    // Write mine geometry
    // Write header
    stream << static_cast<uint8_t>(0);  // Compiled version
    stream << static_cast<uint16_t>(m_vertices.size());
    stream << static_cast<uint16_t>(m_segments.size());
    
    // Write vertices
    for (const auto& vertex : m_vertices) {
        writeVector(stream, vertex.position);
    }
    
    // Write segments
    for (const auto& segment : m_segments) {
        segment.write(stream);
    }
    
    // Remember where game data starts
    qint64 gameDataOffset = file.pos();
    
    // Write game data signature (0x6705)
    stream << static_cast<uint16_t>(0x6705);
    
    // Write level version
    stream << m_levelVersion;
    
    // Write level name (if version >= 14)
    if (m_levelVersion >= 14) {
        const char* name = m_levelName.empty() ? "Untitled" : m_levelName.c_str();
        for (const char* p = name; *p != 0; ++p) {
            stream << static_cast<int8_t>(*p);
        }
        stream << static_cast<int8_t>(0);  // Null terminator
    }
    
    // Write objects
    stream << static_cast<int32_t>(m_objects.size());
    for (const auto& obj : m_objects) {
        obj.write(stream, m_levelVersion);
    }
    
    // Write walls
    stream << static_cast<int32_t>(m_walls.size());
    for (const auto& wall : m_walls) {
        wall.write(stream, m_levelVersion);
    }
    
    // Write triggers
    stream << static_cast<int32_t>(m_triggers.size());
    for (const auto& trigger : m_triggers) {
        trigger.write(stream, false, m_levelVersion);  // false = wall trigger
    }
    
    // Write reactor trigger
    if (m_reactorTrigger.has_value()) {
        stream << static_cast<int32_t>(1);  // Count = 1
        m_reactorTrigger->write(stream);
    } else {
        stream << static_cast<int32_t>(0);  // Count = 0
    }
    
    // Write robot makers
    stream << static_cast<int32_t>(m_robotMakers.size());
    for (const auto& matcen : m_robotMakers) {
        matcen.write(stream, m_levelVersion);
    }
    
    // Write equipment makers (D2 only)
    if (m_levelVersion >= LEVEL_VERSION_D2) {
        stream << static_cast<int32_t>(m_equipmentMakers.size());
        for (const auto& matcen : m_equipmentMakers) {
            matcen.write(stream, m_levelVersion);
        }
        
        // TODO: Write light deltas (D2 only)
    }
    
    // Now go back and write the actual offsets
    qint64 endPos = file.pos();
    file.seek(mineOffsetPos);
    stream << static_cast<int32_t>(mineDataOffset);
    stream << static_cast<int32_t>(gameDataOffset);
    file.seek(endPos);
    
    file.flush();
    m_changesMade = false;
    
    return stream.status() == QDataStream::Ok;
}

} // namespace dle

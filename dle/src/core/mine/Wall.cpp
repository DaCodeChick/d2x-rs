#include "Wall.h"
#include "../io/FileReader.h"
#include "../io/FileWriter.h"

namespace dle {

Wall::Wall()
    : m_type(WallType::None)
    , m_flags(0)
    , m_state(WallState::Closed)
    , m_hitPoints(intToFix(100))
    , m_linkedWall(-1)
    , m_clipNum(-1)
    , m_keys(0)
    , m_trigger(0xFF)
    , m_controllingTrigger(-1)
    , m_cloakValue(0)
{
}

void Wall::clear() {
    m_type = WallType::None;
    m_flags = 0;
    m_state = WallState::Closed;
    m_hitPoints = intToFix(100);
    m_linkedWall = -1;
    m_clipNum = -1;
    m_keys = 0;
    m_trigger = 0xFF;
    m_controllingTrigger = -1;
    m_cloakValue = 0;
}

bool Wall::isDoor() const {
    return m_type == WallType::Door;
}

bool Wall::isVisible() const {
    // Walls that can be seen
    return m_type != WallType::None && 
           m_type != WallType::Open && 
           !(m_type == WallType::Illusion && hasFlag(WallFlagIllusionOff));
}

void Wall::read(FileReader& reader, int levelVersion) {
    // Read wall data based on level version
    // D1 and D2 have slightly different formats
    
    // Read type
    m_type = static_cast<WallType>(reader.readUInt8());
    
    // Read flags
    m_flags = reader.readUInt16();
    
    // Read hit points (as fix)
    m_hitPoints = reader.readFix();
    
    // Read linked wall
    m_linkedWall = reader.readInt16();
    
    // Read state
    m_state = static_cast<WallState>(reader.readUInt8());
    
    // Read trigger
    m_trigger = reader.readUInt8();
    
    // Read clip number
    m_clipNum = reader.readInt8();
    
    // Read keys
    m_keys = reader.readUInt8();
    
    // D2 specific fields
    if (levelVersion > 1) {
        m_controllingTrigger = reader.readInt8();
        m_cloakValue = reader.readInt8();
    } else {
        // D1: These fields were a "short pad"
        reader.readInt16();  // Skip padding
        m_controllingTrigger = -1;
        m_cloakValue = 0;
    }
}

void Wall::write(FileWriter& writer, int levelVersion) const {
    // Write wall data based on level version
    
    // Write type
    writer.writeUInt8(static_cast<uint8_t>(m_type));
    
    // Write flags
    writer.writeUInt16(m_flags);
    
    // Write hit points (as fix)
    writer.writeFix(m_hitPoints);
    
    // Write linked wall
    writer.writeInt16(m_linkedWall);
    
    // Write state
    writer.writeUInt8(static_cast<uint8_t>(m_state));
    
    // Write trigger
    writer.writeUInt8(m_trigger);
    
    // Write clip number
    writer.writeInt8(m_clipNum);
    
    // Write keys
    writer.writeUInt8(m_keys);
    
    // D2 specific fields
    if (levelVersion > 1) {
        writer.writeInt8(m_controllingTrigger);
        writer.writeInt8(m_cloakValue);
    } else {
        // D1: Write padding
        writer.writeInt16(0);
    }
}

} // namespace dle

#include "Wall.h"
#include <QDataStream>

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

void Wall::read(QDataStream& stream, int levelVersion) {
    // Read wall data based on level version
    // D1 and D2 have slightly different formats
    
    uint8_t type_val;
    uint8_t state_val;
    
    // Read type
    stream >> type_val;
    m_type = static_cast<WallType>(type_val);
    
    // Read flags
    stream >> m_flags;
    
    // Read hit points (as fix)
    stream >> m_hitPoints;
    
    // Read linked wall
    stream >> m_linkedWall;
    
    // Read state
    stream >> state_val;
    m_state = static_cast<WallState>(state_val);
    
    // Read trigger
    stream >> m_trigger;
    
    // Read clip number
    stream >> m_clipNum;
    
    // Read keys
    stream >> m_keys;
    
    // D2 specific fields
    if (levelVersion > 1) {
        stream >> m_controllingTrigger;
        stream >> m_cloakValue;
    } else {
        // D1: These fields were a "short pad"
        stream.skipRawData(2);  // Skip padding
        m_controllingTrigger = -1;
        m_cloakValue = 0;
    }
}

void Wall::write(QDataStream& stream, int levelVersion) const {
    // Write wall data based on level version
    
    // Write type
    stream << static_cast<uint8_t>(m_type);
    
    // Write flags
    stream << m_flags;
    
    // Write hit points (as fix)
    stream << m_hitPoints;
    
    // Write linked wall
    stream << m_linkedWall;
    
    // Write state
    stream << static_cast<uint8_t>(m_state);
    
    // Write trigger
    stream << m_trigger;
    
    // Write clip number
    stream << m_clipNum;
    
    // Write keys
    stream << m_keys;
    
    // D2 specific fields
    if (levelVersion > 1) {
        stream << m_controllingTrigger;
        stream << m_cloakValue;
    } else {
        // D1: Write padding
        stream << static_cast<int16_t>(0);
    }
}

} // namespace dle

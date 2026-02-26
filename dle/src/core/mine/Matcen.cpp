#include "Matcen.h"
#include <QDataStream>
#include "../types/Types.h"

namespace dle {

Matcen::Matcen()
    : m_objectFlags{0, 0}
    , m_hitPoints(0)
    , m_interval(0)
    , m_segment(-1)
    , m_fuelCenIndex(-1)
{
}

void Matcen::read(QDataStream& stream, int levelVersion) {
    // Read object flags (robot/equipment types that can spawn)
    stream >> m_objectFlags[0];
    
    // D2 has 64 object types (2 x 32-bit flags), D1 has 32 (1 x 32-bit)
    if (levelVersion >= LEVEL_VERSION_D2) {
        stream >> m_objectFlags[1];
    } else {
        m_objectFlags[1] = 0;
    }
    
    stream >> m_hitPoints;
    stream >> m_interval;
    stream >> m_segment;
    stream >> m_fuelCenIndex;
}

void Matcen::write(QDataStream& stream, int levelVersion) const {
    // Write object flags
    stream << m_objectFlags[0];
    
    // D2 writes both flag sets
    if (levelVersion >= LEVEL_VERSION_D2) {
        stream << m_objectFlags[1];
    }
    
    stream << m_hitPoints;
    stream << m_interval;
    stream << m_segment;
    stream << m_fuelCenIndex;
}

} // namespace dle

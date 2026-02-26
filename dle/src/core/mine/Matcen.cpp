#include "Matcen.h"
#include "../io/FileReader.h"
#include "../io/FileWriter.h"
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

void Matcen::read(FileReader& reader, int levelVersion) {
    // Read object flags (robot/equipment types that can spawn)
    m_objectFlags[0] = reader.readInt32();
    
    // D2 has 64 object types (2 x 32-bit flags), D1 has 32 (1 x 32-bit)
    if (levelVersion >= LEVEL_VERSION_D2) {
        m_objectFlags[1] = reader.readInt32();
    } else {
        m_objectFlags[1] = 0;
    }
    
    m_hitPoints = reader.readInt32();
    m_interval = reader.readInt32();
    m_segment = reader.readInt16();
    m_fuelCenIndex = reader.readInt16();
}

void Matcen::write(FileWriter& writer, int levelVersion) const {
    // Write object flags
    writer.writeInt32(m_objectFlags[0]);
    
    // D2 writes both flag sets
    if (levelVersion >= LEVEL_VERSION_D2) {
        writer.writeInt32(m_objectFlags[1]);
    }
    
    writer.writeInt32(m_hitPoints);
    writer.writeInt32(m_interval);
    writer.writeInt16(m_segment);
    writer.writeInt16(m_fuelCenIndex);
}

} // namespace dle

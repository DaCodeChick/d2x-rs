#pragma once

#include <cstdint>
#include <array>

class QDataStream;  // Qt forward declaration

namespace dle {

/**
 * @brief Matcen (Materialization Center) - Robot/equipment generator
 * 
 * Matcens spawn robots or equipment in Descent levels. There are two types:
 * - Robot makers (spawn enemies)
 * - Equipment makers (spawn powerups, D2 only)
 * 
 * Each matcen is attached to a segment and can spawn up to 32 different
 * object types (64 in D2).
 */
class Matcen {
public:
    Matcen();
    ~Matcen() = default;
    
    // Copy/move constructors
    Matcen(const Matcen&) = default;
    Matcen& operator=(const Matcen&) = default;
    Matcen(Matcen&&) = default;
    Matcen& operator=(Matcen&&) = default;
    
    // Accessors
    const std::array<int32_t, 2>& getObjectFlags() const { return m_objectFlags; }
    void setObjectFlags(const std::array<int32_t, 2>& flags) { m_objectFlags = flags; }
    
    int32_t getHitPoints() const { return m_hitPoints; }
    void setHitPoints(int32_t hp) { m_hitPoints = hp; }
    
    int32_t getInterval() const { return m_interval; }
    void setInterval(int32_t interval) { m_interval = interval; }
    
    int16_t getSegment() const { return m_segment; }
    void setSegment(int16_t segment) { m_segment = segment; }
    
    int16_t getFuelCenIndex() const { return m_fuelCenIndex; }
    void setFuelCenIndex(int16_t index) { m_fuelCenIndex = index; }
    
    // Check if specific robot/object type is enabled
    bool hasObjectType(int objectId) const {
        if (objectId < 0 || objectId >= 64) return false;
        int arrayIndex = objectId / 32;
        int bitIndex = objectId % 32;
        return (m_objectFlags[arrayIndex] & (1 << bitIndex)) != 0;
    }
    
    // Set/clear specific robot/object type
    void setObjectType(int objectId, bool enabled) {
        if (objectId < 0 || objectId >= 64) return;
        int arrayIndex = objectId / 32;
        int bitIndex = objectId % 32;
        if (enabled) {
            m_objectFlags[arrayIndex] |= (1 << bitIndex);
        } else {
            m_objectFlags[arrayIndex] &= ~(1 << bitIndex);
        }
    }
    
    // File I/O
    void read(QDataStream& stream, int levelVersion);
    void write(QDataStream& stream, int levelVersion) const;
    
private:
    std::array<int32_t, 2> m_objectFlags;  // Bitmask for up to 64 object types
    int32_t m_hitPoints;                   // How hard it is to destroy
    int32_t m_interval;                    // Time between materializations
    int16_t m_segment;                     // Segment this is attached to
    int16_t m_fuelCenIndex;                // Index in fuelcen array (or producer index)
};

} // namespace dle

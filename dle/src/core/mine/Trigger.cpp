#include "Trigger.h"
#include "../io/FileReader.h"
#include "../io/FileWriter.h"
#include <algorithm>

namespace dle {

// ===== Trigger =====

Trigger::Trigger()
    : m_type(TriggerType::OpenDoor)
    , m_flags(0)
    , m_value(0)
    , m_time(0)
    , m_object(-1)
    , m_targetCount(0)
{
}

void Trigger::clear() {
    m_type = TriggerType::OpenDoor;
    m_flags = 0;
    m_value = 0;
    m_time = 0;
    m_object = -1;
    clearTargets();
}

bool Trigger::addTarget(int16_t segmentId, int16_t sideId) {
    if (m_targetCount >= MAX_TRIGGER_TARGETS) {
        return false;
    }
    
    m_targets[m_targetCount] = TriggerTarget(segmentId, sideId);
    m_targetCount++;
    return true;
}

bool Trigger::addTarget(const TriggerTarget& target) {
    return addTarget(target.segmentId, target.sideId);
}

void Trigger::removeTarget(int index) {
    if (index < 0 || index >= m_targetCount) {
        return;
    }
    
    // Shift targets down
    for (int i = index; i < m_targetCount - 1; ++i) {
        m_targets[i] = m_targets[i + 1];
    }
    
    m_targetCount--;
}

void Trigger::clearTargets() {
    m_targetCount = 0;
    for (auto& target : m_targets) {
        target.segmentId = -1;
        target.sideId = -1;
    }
}

void Trigger::read(FileReader& reader, bool isObjectTrigger, int levelVersion) {
    // Read trigger type
    m_type = static_cast<TriggerType>(reader.readUInt8());
    
    // Read flags
    m_flags = reader.readUInt16();
    
    // Read value (as fix)
    m_value = reader.readFix();
    
    // Read time (as fix)
    m_time = reader.readFix();
    
    // Read number of targets
    m_targetCount = std::min(reader.readInt8(), static_cast<int8_t>(MAX_TRIGGER_TARGETS));
    
    // Read object index (for object triggers)
    if (isObjectTrigger) {
        m_object = reader.readInt16();
    } else {
        m_object = -1;
    }
    
    // Read targets
    for (int i = 0; i < m_targetCount; ++i) {
        m_targets[i].segmentId = reader.readInt16();
        m_targets[i].sideId = reader.readInt16();
    }
}

void Trigger::write(FileWriter& writer, bool isObjectTrigger, int levelVersion) const {
    // Write trigger type
    writer.writeUInt8(static_cast<uint8_t>(m_type));
    
    // Write flags
    writer.writeUInt16(m_flags);
    
    // Write value (as fix)
    writer.writeFix(m_value);
    
    // Write time (as fix)
    writer.writeFix(m_time);
    
    // Write number of targets
    writer.writeInt8(m_targetCount);
    
    // Write object index (for object triggers)
    if (isObjectTrigger) {
        writer.writeInt16(m_object);
    }
    
    // Write targets
    for (int i = 0; i < m_targetCount; ++i) {
        writer.writeInt16(m_targets[i].segmentId);
        writer.writeInt16(m_targets[i].sideId);
    }
}

// ===== ReactorTrigger =====

ReactorTrigger::ReactorTrigger()
    : m_targetCount(0)
{
}

void ReactorTrigger::clear() {
    clearTargets();
}

bool ReactorTrigger::addTarget(int16_t segmentId, int16_t sideId) {
    if (m_targetCount >= MAX_TRIGGER_TARGETS) {
        return false;
    }
    
    m_targets[m_targetCount] = TriggerTarget(segmentId, sideId);
    m_targetCount++;
    return true;
}

bool ReactorTrigger::addTarget(const TriggerTarget& target) {
    return addTarget(target.segmentId, target.sideId);
}

void ReactorTrigger::removeTarget(int index) {
    if (index < 0 || index >= m_targetCount) {
        return;
    }
    
    // Shift targets down
    for (int i = index; i < m_targetCount - 1; ++i) {
        m_targets[i] = m_targets[i + 1];
    }
    
    m_targetCount--;
}

void ReactorTrigger::clearTargets() {
    m_targetCount = 0;
    for (auto& target : m_targets) {
        target.segmentId = -1;
        target.sideId = -1;
    }
}

void ReactorTrigger::read(FileReader& reader) {
    // Read number of targets
    m_targetCount = std::min(reader.readInt8(), static_cast<int8_t>(MAX_TRIGGER_TARGETS));
    
    // Read targets
    for (int i = 0; i < m_targetCount; ++i) {
        m_targets[i].segmentId = reader.readInt16();
        m_targets[i].sideId = reader.readInt16();
    }
}

void ReactorTrigger::write(FileWriter& writer) const {
    // Write number of targets
    writer.writeInt8(m_targetCount);
    
    // Write targets
    for (int i = 0; i < m_targetCount; ++i) {
        writer.writeInt16(m_targets[i].segmentId);
        writer.writeInt16(m_targets[i].sideId);
    }
}

} // namespace dle

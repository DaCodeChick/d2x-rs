#include "Trigger.h"
#include <QDataStream>
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

void Trigger::read(QDataStream& stream, bool isObjectTrigger, int levelVersion) {
    uint8_t type_val;
    int8_t targetCount;
    
    // Read trigger type
    stream >> type_val;
    m_type = static_cast<TriggerType>(type_val);
    
    // Read flags
    stream >> m_flags;
    
    // Read value (as fix)
    stream >> m_value;
    
    // Read time (as fix)
    stream >> m_time;
    
    // Read number of targets
    stream >> targetCount;
    m_targetCount = std::min(targetCount, static_cast<int8_t>(MAX_TRIGGER_TARGETS));
    
    // Read object index (for object triggers)
    if (isObjectTrigger) {
        stream >> m_object;
    } else {
        m_object = -1;
    }
    
    // Read targets
    for (int i = 0; i < m_targetCount; ++i) {
        stream >> m_targets[i].segmentId;
        stream >> m_targets[i].sideId;
    }
}

void Trigger::write(QDataStream& stream, bool isObjectTrigger, int levelVersion) const {
    // Write trigger type
    stream << static_cast<uint8_t>(m_type);
    
    // Write flags
    stream << m_flags;
    
    // Write value (as fix)
    stream << m_value;
    
    // Write time (as fix)
    stream << m_time;
    
    // Write number of targets
    stream << m_targetCount;
    
    // Write object index (for object triggers)
    if (isObjectTrigger) {
        stream << m_object;
    }
    
    // Write targets
    for (int i = 0; i < m_targetCount; ++i) {
        stream << m_targets[i].segmentId;
        stream << m_targets[i].sideId;
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

void ReactorTrigger::read(QDataStream& stream) {
    int8_t targetCount;
    
    // Read number of targets
    stream >> targetCount;
    m_targetCount = std::min(targetCount, static_cast<int8_t>(MAX_TRIGGER_TARGETS));
    
    // Read targets
    for (int i = 0; i < m_targetCount; ++i) {
        stream >> m_targets[i].segmentId;
        stream >> m_targets[i].sideId;
    }
}

void ReactorTrigger::write(QDataStream& stream) const {
    // Write number of targets
    stream << m_targetCount;
    
    // Write targets
    for (int i = 0; i < m_targetCount; ++i) {
        stream << m_targets[i].segmentId;
        stream << m_targets[i].sideId;
    }
}

} // namespace dle

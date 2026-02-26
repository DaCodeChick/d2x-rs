#ifndef DLE_TRIGGER_H
#define DLE_TRIGGER_H

#include "../types/Types.h"
#include <cstdint>
#include <array>

namespace dle {

// Forward declaration
class FileReader;
class FileWriter;

/**
 * @brief Maximum number of trigger targets
 */
constexpr int MAX_TRIGGER_TARGETS = 10;

/**
 * @brief Trigger types
 */
enum class TriggerType : uint8_t {
    OpenDoor = 0,          // Open door
    CloseDoor = 1,         // Close door (unused in D1/D2)
    Matcen = 2,            // Activate matcen
    Exit = 3,              // Exit level
    SecretExit = 4,        // Secret exit
    IllusionOff = 5,       // Turn off illusion
    IllusionOn = 6,        // Turn on illusion
    UnlockDoor = 7,        // Unlock door
    LockDoor = 8,          // Lock door
    OpenWall = 9,          // Open wall (unused)
    CloseWall = 10,        // Close wall (unused)
    IllusoryWall = 11,     // Illusory wall (unused)
    LightOff = 12,         // Turn lights off
    LightOn = 13,          // Turn lights on
    
    // D2X-XL extensions
    Teleport = 14,         // Teleport player
    SpeedBoost = 15,       // Speed boost
    CameraOff = 16,        // Turn camera off
    CameraOn = 17,         // Turn camera on
    ShieldDamage = 18,     // Damage shields
    EnergyDrain = 19,      // Drain energy
    ChangeTexture = 20,    // Change wall texture
    SetSpawn = 21,         // Set spawn point
    Message = 22,          // Display message
    Sound = 23,            // Play sound
    MasterTrigger = 24,    // Master trigger (activates other triggers)
    EnterLevel = 25,       // Trigger on level entry
};

/**
 * @brief Trigger flags
 */
enum TriggerFlags : uint16_t {
    TriggerNoMessage = 0x0001,      // Don't show messages
    TriggerOneShot = 0x0002,        // Trigger once only
    TriggerDisabled = 0x0004,       // Trigger is disabled
    TriggerOn = 0x0008,             // Trigger is on
    TriggerPermanent = 0x0010,      // Permanent trigger
    TriggerAlternate = 0x0020,      // Alternate between on/off
    TriggerSetOrient = 0x0040,      // Set orientation on teleport
    TriggerAutoplay = 0x0080,       // Autoplay (object trigger)
};

/**
 * @brief Trigger target (segment and side)
 */
struct TriggerTarget {
    int16_t segmentId;
    int16_t sideId;
    
    TriggerTarget() : segmentId(-1), sideId(-1) {}
    TriggerTarget(int16_t seg, int16_t side) : segmentId(seg), sideId(side) {}
    
    bool isValid() const { return segmentId >= 0 && sideId >= 0; }
};

/**
 * @brief Trigger represents an event in the level
 * 
 * Triggers are activated by various conditions (shooting, collision, etc.)
 * and perform actions on target segments/sides (open doors, activate matcens, etc.)
 */
class Trigger {
public:
    Trigger();
    ~Trigger() = default;
    
    // Copy/move constructors
    Trigger(const Trigger&) = default;
    Trigger& operator=(const Trigger&) = default;
    Trigger(Trigger&&) = default;
    Trigger& operator=(Trigger&&) = default;
    
    void clear();
    
    // Type and properties
    TriggerType getType() const { return m_type; }
    void setType(TriggerType type) { m_type = type; }
    
    uint16_t getFlags() const { return m_flags; }
    void setFlags(uint16_t flags) { m_flags = flags; }
    bool hasFlag(TriggerFlags flag) const { return (m_flags & flag) != 0; }
    void addFlag(TriggerFlags flag) { m_flags |= flag; }
    void removeFlag(TriggerFlags flag) { m_flags &= ~flag; }
    
    // Value (function-specific: time, damage amount, etc.)
    fix getValue() const { return m_value; }
    void setValue(fix value) { m_value = value; }
    
    // Time (for timed triggers)
    fix getTime() const { return m_time; }
    void setTime(fix time) { m_time = time; }
    
    // Object (for object triggers, -1 = wall trigger)
    int16_t getObject() const { return m_object; }
    void setObject(int16_t obj) { m_object = obj; }
    
    // Targets
    int getTargetCount() const { return m_targetCount; }
    
    const TriggerTarget& getTarget(int index) const { 
        return m_targets[index]; 
    }
    
    TriggerTarget& getTarget(int index) { 
        return m_targets[index]; 
    }
    
    bool addTarget(int16_t segmentId, int16_t sideId);
    bool addTarget(const TriggerTarget& target);
    void removeTarget(int index);
    void clearTargets();
    
    // Check trigger type
    bool isExit() const { 
        return m_type == TriggerType::Exit || m_type == TriggerType::SecretExit; 
    }
    
    bool isObjectTrigger() const { return m_object >= 0; }
    bool isWallTrigger() const { return m_object < 0; }
    bool isD2X() const { return m_type >= TriggerType::Teleport; }
    
    // File I/O
    void read(FileReader& reader, bool isObjectTrigger, int levelVersion);
    void write(FileWriter& writer, bool isObjectTrigger, int levelVersion) const;

private:
    TriggerType m_type;                                      // Trigger type
    uint16_t m_flags;                                        // Trigger flags
    fix m_value;                                             // Function-specific value
    fix m_time;                                              // Time (for timed triggers)
    int16_t m_object;                                        // Object index (-1 = wall trigger)
    
    // Targets
    int8_t m_targetCount;                                    // Number of targets
    std::array<TriggerTarget, MAX_TRIGGER_TARGETS> m_targets; // Target list
};

/**
 * @brief ReactorTrigger represents targets activated when reactor is destroyed
 */
class ReactorTrigger {
public:
    ReactorTrigger();
    ~ReactorTrigger() = default;
    
    // Copy/move constructors
    ReactorTrigger(const ReactorTrigger&) = default;
    ReactorTrigger& operator=(const ReactorTrigger&) = default;
    ReactorTrigger(ReactorTrigger&&) = default;
    ReactorTrigger& operator=(ReactorTrigger&&) = default;
    
    void clear();
    
    // Targets
    int getTargetCount() const { return m_targetCount; }
    
    const TriggerTarget& getTarget(int index) const { 
        return m_targets[index]; 
    }
    
    TriggerTarget& getTarget(int index) { 
        return m_targets[index]; 
    }
    
    bool addTarget(int16_t segmentId, int16_t sideId);
    bool addTarget(const TriggerTarget& target);
    void removeTarget(int index);
    void clearTargets();
    
    // File I/O
    void read(FileReader& reader);
    void write(FileWriter& writer) const;

private:
    int8_t m_targetCount;                                    // Number of targets
    std::array<TriggerTarget, MAX_TRIGGER_TARGETS> m_targets; // Target list
};

} // namespace dle

#endif // DLE_TRIGGER_H

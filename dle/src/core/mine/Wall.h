#ifndef DLE_WALL_H
#define DLE_WALL_H

#include "../types/Types.h"
#include <cstdint>

class QDataStream;  // Qt forward declaration

namespace dle {

/**
 * @brief Wall types
 */
enum class WallType : uint8_t {
    None = 0,
    Normal = 1,          // Normal destructible wall
    Blastable = 2,       // Blastable wall (shootable)
    Door = 3,            // Standard door (slides open)
    Illusion = 4,        // Illusory wall (walk through)
    Open = 5,            // Open wall (no collision)
    Closed = 6,          // Closed wall (solid, not destructible)
    Overlay = 7,         // Overlay (transparent)
    Cloaked = 8,         // Cloaked wall (invisible)
    
    // D2X-XL extensions
    Colored = 9,         // Colored transparent wall (D2X-XL)
};

/**
 * @brief Wall flags
 */
enum WallFlags : uint16_t {
    WallFlagBlasted = 0x0001,          // Wall has been blasted
    WallFlagDoorOpened = 0x0002,       // Door is opened
    WallFlagDoorLocked = 0x0008,       // Door is locked
    WallFlagDoorAuto = 0x0010,         // Door closes automatically
    WallFlagIllusionOff = 0x0020,      // Illusion is off
    WallFlagSwitchBlasted = 0x0040,    // Switch wall has been shot
    WallFlagBuddyProof = 0x0100,       // Buddy bot proof
    WallFlagIgnoreMarker = 0x0200,     // Ignore marker messages
    WallFlagSetOrient = 0x0400,        // Set orientation flag
};

/**
 * @brief Wall state
 */
enum class WallState : uint8_t {
    Opening = 0,         // Door is opening
    Closed = 1,          // Door is closed
    Open = 2,            // Door is open
    Closing = 3,         // Door is closing
    Cloaking = 4,        // Wall is cloaking
    Decloaking = 5,      // Wall is decloaking
};

/**
 * @brief Wall keys (for locked doors)
 */
enum WallKeys : uint8_t {
    KeyNone = 0x00,
    KeyBlue = 0x01,
    KeyRed = 0x02,
    KeyGold = 0x04,
};

/**
 * @brief Wall represents a wall/door in a segment side
 * 
 * Walls are special properties attached to segment sides.
 * They can be doors, destructible walls, illusions, etc.
 */
class Wall {
public:
    Wall();
    ~Wall() = default;
    
    // Copy/move constructors
    Wall(const Wall&) = default;
    Wall& operator=(const Wall&) = default;
    Wall(Wall&&) = default;
    Wall& operator=(Wall&&) = default;
    
    void clear();
    
    // Type and properties
    WallType getType() const { return m_type; }
    void setType(WallType type) { m_type = type; }
    
    uint16_t getFlags() const { return m_flags; }
    void setFlags(uint16_t flags) { m_flags = flags; }
    bool hasFlag(WallFlags flag) const { return (m_flags & flag) != 0; }
    void addFlag(WallFlags flag) { m_flags |= flag; }
    void removeFlag(WallFlags flag) { m_flags &= ~flag; }
    
    WallState getState() const { return m_state; }
    void setState(WallState state) { m_state = state; }
    
    // Hit points (strength)
    fix getHitPoints() const { return m_hitPoints; }
    void setHitPoints(fix hp) { m_hitPoints = hp; }
    
    // Linked wall (for multi-part walls)
    int16_t getLinkedWall() const { return m_linkedWall; }
    void setLinkedWall(int16_t wall) { m_linkedWall = wall; }
    
    // Animation clip
    int8_t getClipNum() const { return m_clipNum; }
    void setClipNum(int8_t clip) { m_clipNum = clip; }
    
    // Keys required
    uint8_t getKeys() const { return m_keys; }
    void setKeys(uint8_t keys) { m_keys = keys; }
    bool requiresKey(WallKeys key) const { return (m_keys & key) != 0; }
    
    // Trigger
    uint8_t getTrigger() const { return m_trigger; }
    void setTrigger(uint8_t trigger) { m_trigger = trigger; }
    
    // D2 specific
    int8_t getControllingTrigger() const { return m_controllingTrigger; }
    void setControllingTrigger(int8_t trigger) { m_controllingTrigger = trigger; }
    
    int8_t getCloakValue() const { return m_cloakValue; }
    void setCloakValue(int8_t value) { m_cloakValue = value; }
    
    // Check wall type
    bool isDoor() const;
    bool isVisible() const;
    bool isTransparent() const { return m_type == WallType::Colored; }
    bool isCloaked() const { return m_type == WallType::Cloaked; }
    bool isIllusion() const { return m_type == WallType::Illusion; }
    bool isClosed() const { return m_type == WallType::Closed; }
    bool isD2X() const { return m_type >= WallType::Colored; }
    
    // File I/O
    void read(QDataStream& stream, int levelVersion);
    void write(QDataStream& stream, int levelVersion) const;

private:
    WallType m_type;                  // Wall type
    uint16_t m_flags;                 // Wall flags
    WallState m_state;                // Current state (opening, closing, etc.)
    fix m_hitPoints;                  // Hit points (strength)
    int16_t m_linkedWall;             // Linked wall index (-1 = none)
    int8_t m_clipNum;                 // Animation clip index (-1 = none)
    uint8_t m_keys;                   // Keys required (bitfield)
    uint8_t m_trigger;                // Trigger index (0xFF = none)
    int8_t m_controllingTrigger;      // D2: Controlling trigger (-1 = none)
    int8_t m_cloakValue;              // D2: Cloak fade value (0-31)
};

} // namespace dle

#endif // DLE_WALL_H

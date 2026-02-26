#include "Object.h"
#include "../io/FileReader.h"
#include "../io/FileWriter.h"

namespace dle {

// ===== PhysicsInfo =====

PhysicsInfo::PhysicsInfo()
    : velocity()
    , thrust()
    , mass(intToFix(1))
    , drag(0)
    , brakes(0)
    , rotVelocity()
    , rotThrust()
    , turnRoll(0)
    , flags(0)
{
}

void PhysicsInfo::clear() {
    velocity = Vector();
    thrust = Vector();
    mass = intToFix(1);
    drag = 0;
    brakes = 0;
    rotVelocity = Vector();
    rotThrust = Vector();
    turnRoll = 0;
    flags = 0;
}

void PhysicsInfo::read(FileReader& reader) {
    velocity = reader.readVector();
    thrust = reader.readVector();
    mass = reader.readFix();
    drag = reader.readFix();
    brakes = reader.readFix();
    rotVelocity = reader.readVector();
    rotThrust = reader.readVector();
    turnRoll = reader.readInt16();
    flags = reader.readUInt16();
}

void PhysicsInfo::write(FileWriter& writer) const {
    writer.writeVector(velocity);
    writer.writeVector(thrust);
    writer.writeFix(mass);
    writer.writeFix(drag);
    writer.writeFix(brakes);
    writer.writeVector(rotVelocity);
    writer.writeVector(rotThrust);
    writer.writeInt16(turnRoll);
    writer.writeUInt16(flags);
}

// ===== AIInfo =====

AIInfo::AIInfo()
    : behavior(0)
    , hideSegment(-1)
    , pathLength(0)
{
}

void AIInfo::clear() {
    behavior = 0;
    hideSegment = -1;
    pathLength = 0;
}

void AIInfo::read(FileReader& reader) {
    behavior = reader.readUInt8();
    
    // Skip AI flags (11 bytes)
    for (int i = 0; i < 11; ++i) {
        reader.readInt8();
    }
    
    hideSegment = reader.readInt16();
    
    // Skip hide_index, cur_path_index (4 bytes)
    reader.readInt16();
    
    pathLength = reader.readInt16();
    
    // Skip cur_path_index (2 bytes)
    reader.readInt16();
    
    // Skip follow_path_start_seg, follow_path_end_seg (4 bytes)
    reader.readInt16();
    reader.readInt16();
    
    // Skip danger_laser_signature, danger_laser_num (6 bytes)
    reader.readInt32();
    reader.readInt16();
}

void AIInfo::write(FileWriter& writer) const {
    writer.writeUInt8(behavior);
    
    // Write AI flags (11 bytes of zeros)
    for (int i = 0; i < 11; ++i) {
        writer.writeInt8(0);
    }
    
    writer.writeInt16(hideSegment);
    writer.writeInt16(0);  // hide_index
    writer.writeInt16(pathLength);
    writer.writeInt16(0);  // cur_path_index
    writer.writeInt16(0);  // follow_path_start_seg
    writer.writeInt16(0);  // follow_path_end_seg
    writer.writeInt32(0);  // danger_laser_signature
    writer.writeInt16(0);  // danger_laser_num
}

// ===== PowerupInfo =====

PowerupInfo::PowerupInfo()
    : count(0)
{
}

void PowerupInfo::clear() {
    count = 0;
}

void PowerupInfo::read(FileReader& reader) {
    count = reader.readInt32();
}

void PowerupInfo::write(FileWriter& writer) const {
    writer.writeInt32(count);
}

// ===== ObjectContents =====

ObjectContents::ObjectContents()
    : type(-1)
    , id(-1)
    , count(0)
{
}

// ===== Object =====

Object::Object()
    : m_signature(0)
    , m_type(ObjectType::None)
    , m_id(0)
    , m_controlType(ControlType::None)
    , m_movementType(MovementType::None)
    , m_renderType(RenderType::None)
    , m_flags(0)
    , m_multiplayer(0)
    , m_segment(-1)
    , m_position()
    , m_lastPosition()
    , m_orientation()
    , m_size(intToFix(1))
    , m_shields(0)
    , m_contents()
    , m_physicsInfo()
    , m_spinRate()
    , m_aiInfo()
    , m_powerupInfo()
{
}

void Object::clear() {
    m_signature = 0;
    m_type = ObjectType::None;
    m_id = 0;
    m_controlType = ControlType::None;
    m_movementType = MovementType::None;
    m_renderType = RenderType::None;
    m_flags = 0;
    m_multiplayer = 0;
    m_segment = -1;
    m_position = Vector();
    m_lastPosition = Vector();
    m_orientation = Matrix();
    m_size = intToFix(1);
    m_shields = 0;
    m_contents = ObjectContents();
    m_physicsInfo.clear();
    m_spinRate = Vector();
    m_aiInfo.clear();
    m_powerupInfo.clear();
}

void Object::read(FileReader& reader, int levelVersion) {
    // Read basic object info
    m_type = static_cast<ObjectType>(reader.readInt8());
    m_id = reader.readInt8();
    m_controlType = static_cast<ControlType>(reader.readInt8());
    m_movementType = static_cast<MovementType>(reader.readInt8());
    m_renderType = static_cast<RenderType>(reader.readInt8());
    m_flags = reader.readUInt8();
    
    // Multiplayer flag (version > 37 in D2X-XL)
    if (levelVersion > 37) {
        m_multiplayer = reader.readUInt8();
    } else {
        m_multiplayer = 0;
    }
    
    m_segment = reader.readInt16();
    
    // Position and orientation
    m_position = reader.readVector();
    m_orientation = reader.readMatrix();
    
    m_size = reader.readFix();
    m_shields = reader.readFix();
    
    // Last position
    m_lastPosition = reader.readVector();
    
    // Contents (what's inside when destroyed)
    m_contents.type = reader.readInt8();
    m_contents.id = reader.readInt8();
    m_contents.count = reader.readInt8();
    
    // Read movement type specific data
    switch (m_movementType) {
        case MovementType::Physics:
            m_physicsInfo.read(reader);
            break;
            
        case MovementType::Spinning:
            m_spinRate = reader.readVector();
            break;
            
        case MovementType::None:
        default:
            break;
    }
    
    // Read control type specific data
    switch (m_controlType) {
        case ControlType::AI:
            m_aiInfo.read(reader);
            break;
            
        case ControlType::Powerup:
            m_powerupInfo.read(reader);
            break;
            
        case ControlType::Explosion:
            // Skip explosion info (16 bytes)
            reader.readInt32();  // spawn_time
            reader.readInt32();  // delete_time
            reader.readInt8();   // delete_objnum
            reader.readInt8();   // attach_parent
            reader.readInt8();   // prev_attach
            reader.readInt8();   // next_attach
            break;
            
        case ControlType::Weapon:
            // Skip weapon/laser info (16 bytes)
            reader.readInt16();  // parent_type
            reader.readInt16();  // parent_num
            reader.readInt32();  // parent_signature
            reader.readInt32();  // creation_time
            reader.readInt8();   // last_hitobj
            reader.readInt8();   // track_goal
            reader.readInt16();  // padding
            reader.readInt32();  // multiplier
            break;
            
        case ControlType::Light:
            // Skip light info (4 bytes)
            reader.readInt32();  // intensity
            break;
            
        case ControlType::None:
        case ControlType::Flying:
        case ControlType::Slew:
        case ControlType::Debris:
        case ControlType::ControlCenter:
        default:
            break;
    }
    
    // Read render type specific data
    switch (m_renderType) {
        case RenderType::Polymodel:
            // Skip polymodel info (variable size, read minimal)
            reader.readInt32();  // model number
            for (int i = 0; i < 10; ++i) {
                reader.readInt16();  // anim_angles (10 submodels * 3 angles * 2 bytes = 60 bytes, but stored as fixang)
                reader.readInt16();
                reader.readInt16();
            }
            reader.readInt32();  // subobj_flags
            reader.readInt32();  // tmap_override
            reader.readInt8();   // alt_textures
            break;
            
        case RenderType::Fireball:
        case RenderType::VClip:
        case RenderType::WeaponVClip:
            // Skip vclip info (9 bytes)
            reader.readInt32();  // vclip_num
            reader.readInt32();  // frametime
            reader.readInt8();   // framenum
            break;
            
        case RenderType::None:
        case RenderType::Laser:
        case RenderType::Hostage:
        case RenderType::Powerup:
        case RenderType::Morphing:
        case RenderType::Debris:
        case RenderType::Smoke:
        default:
            break;
    }
}

void Object::write(FileWriter& writer, int levelVersion) const {
    // Write basic object info
    writer.writeInt8(static_cast<int8_t>(m_type));
    writer.writeInt8(m_id);
    writer.writeInt8(static_cast<int8_t>(m_controlType));
    writer.writeInt8(static_cast<int8_t>(m_movementType));
    writer.writeInt8(static_cast<int8_t>(m_renderType));
    writer.writeUInt8(m_flags);
    
    // Multiplayer flag (version > 37)
    if (levelVersion > 37) {
        writer.writeUInt8(m_multiplayer);
    }
    
    writer.writeInt16(m_segment);
    
    // Position and orientation
    writer.writeVector(m_position);
    writer.writeMatrix(m_orientation);
    
    writer.writeFix(m_size);
    writer.writeFix(m_shields);
    
    // Last position
    writer.writeVector(m_lastPosition);
    
    // Contents
    writer.writeInt8(m_contents.type);
    writer.writeInt8(m_contents.id);
    writer.writeInt8(m_contents.count);
    
    // Write movement type specific data
    switch (m_movementType) {
        case MovementType::Physics:
            m_physicsInfo.write(writer);
            break;
            
        case MovementType::Spinning:
            writer.writeVector(m_spinRate);
            break;
            
        case MovementType::None:
        default:
            break;
    }
    
    // Write control type specific data
    switch (m_controlType) {
        case ControlType::AI:
            m_aiInfo.write(writer);
            break;
            
        case ControlType::Powerup:
            m_powerupInfo.write(writer);
            break;
            
        case ControlType::Explosion:
            // Write explosion info (16 bytes of zeros)
            writer.writeInt32(0);  // spawn_time
            writer.writeInt32(0);  // delete_time
            writer.writeInt8(0);   // delete_objnum
            writer.writeInt8(0);   // attach_parent
            writer.writeInt8(0);   // prev_attach
            writer.writeInt8(0);   // next_attach
            break;
            
        case ControlType::Weapon:
            // Write weapon/laser info (16 bytes of zeros)
            writer.writeInt16(0);  // parent_type
            writer.writeInt16(0);  // parent_num
            writer.writeInt32(0);  // parent_signature
            writer.writeInt32(0);  // creation_time
            writer.writeInt8(0);   // last_hitobj
            writer.writeInt8(0);   // track_goal
            writer.writeInt16(0);  // padding
            writer.writeInt32(0);  // multiplier
            break;
            
        case ControlType::Light:
            // Write light info (4 bytes)
            writer.writeInt32(0);  // intensity
            break;
            
        case ControlType::None:
        case ControlType::Flying:
        case ControlType::Slew:
        case ControlType::Debris:
        case ControlType::ControlCenter:
        default:
            break;
    }
    
    // Write render type specific data
    switch (m_renderType) {
        case RenderType::Polymodel:
            // Write polymodel info (minimal)
            writer.writeInt32(m_id);  // model number
            for (int i = 0; i < 10; ++i) {
                writer.writeInt16(0);  // anim_angles
                writer.writeInt16(0);
                writer.writeInt16(0);
            }
            writer.writeInt32(0);  // subobj_flags
            writer.writeInt32(-1); // tmap_override
            writer.writeInt8(0);   // alt_textures
            break;
            
        case RenderType::Fireball:
        case RenderType::VClip:
        case RenderType::WeaponVClip:
            // Write vclip info
            writer.writeInt32(m_id);  // vclip_num
            writer.writeInt32(0);     // frametime
            writer.writeInt8(0);      // framenum
            break;
            
        case RenderType::None:
        case RenderType::Laser:
        case RenderType::Hostage:
        case RenderType::Powerup:
        case RenderType::Morphing:
        case RenderType::Debris:
        case RenderType::Smoke:
        default:
            break;
    }
}

} // namespace dle

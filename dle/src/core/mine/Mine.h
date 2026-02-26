#ifndef DLE_MINE_H
#define DLE_MINE_H

#include "../types/Types.h"
#include "Segment.h"
#include <vector>
#include <string>
#include <memory>

namespace dle {

// Forward declarations
class Wall;
class Trigger;
class Object;
class Reactor;
class Matcen;

/**
 * @brief Vertex represents a 3D point in the mine
 */
struct Vertex {
    Vector position;
    
    Vertex() : position() {}
    Vertex(fix x, fix y, fix z) : position(x, y, z) {}
    Vertex(const Vector& pos) : position(pos) {}
};

/**
 * @brief Mine represents a complete Descent level
 * 
 * Contains all the data for a level:
 * - Vertices (3D points)
 * - Segments (cubes)
 * - Walls (doors, destructible walls, etc.)
 * - Triggers (events)
 * - Objects (robots, powerups, players, etc.)
 * - Matcens (robot generators)
 * - Reactor/boss data
 */
class Mine {
public:
    Mine();
    ~Mine() = default;
    
    // Copy/move constructors
    Mine(const Mine&) = default;
    Mine& operator=(const Mine&) = default;
    Mine(Mine&&) = default;
    Mine& operator=(Mine&&) = default;
    
    // Initialization
    void clear();
    void initialize();
    void reset();
    
    // File type and version
    FileType getFileType() const { return m_fileType; }
    void setFileType(FileType type) { m_fileType = type; }
    
    int getLevelVersion() const { return m_levelVersion; }
    void setLevelVersion(int version) { m_levelVersion = version; }
    
    bool isD1() const { return m_fileType == FileType::RDL; }
    bool isD2() const { return m_fileType == FileType::RL2; }
    bool isD2X() const { return m_fileType == FileType::D2X_XL || m_levelVersion >= LEVEL_VERSION_D2X; }
    bool isStandard() const { return !isD2X(); }
    
    // Level name
    const std::string& getLevelName() const { return m_levelName; }
    void setLevelName(const std::string& name) { m_levelName = name; }
    
    // Vertices
    std::vector<Vertex>& getVertices() { return m_vertices; }
    const std::vector<Vertex>& getVertices() const { return m_vertices; }
    int getVertexCount() const { return static_cast<int>(m_vertices.size()); }
    
    Vertex& getVertex(int index) { return m_vertices[index]; }
    const Vertex& getVertex(int index) const { return m_vertices[index]; }
    
    int addVertex(const Vertex& vertex) {
        m_vertices.push_back(vertex);
        return static_cast<int>(m_vertices.size()) - 1;
    }
    
    int addVertex(fix x, fix y, fix z) {
        return addVertex(Vertex(x, y, z));
    }
    
    void removeVertex(int index);
    
    // Segments
    std::vector<Segment>& getSegments() { return m_segments; }
    const std::vector<Segment>& getSegments() const { return m_segments; }
    int getSegmentCount() const { return static_cast<int>(m_segments.size()); }
    
    Segment& getSegment(int index) { return m_segments[index]; }
    const Segment& getSegment(int index) const { return m_segments[index]; }
    
    int addSegment(const Segment& segment) {
        m_segments.push_back(segment);
        return static_cast<int>(m_segments.size()) - 1;
    }
    
    void removeSegment(int index);
    
    // Limits based on file type
    int getMaxSegments() const {
        if (isD2X()) return MAX_SEGMENTS_D2X;
        return isD1() ? MAX_SEGMENTS_D1 : MAX_SEGMENTS_D2;
    }
    
    int getMaxVertices() const {
        if (isD2X()) return MAX_VERTICES_D2X;
        return isD1() ? MAX_VERTICES_D1 : MAX_VERTICES_D2;
    }
    
    int getMaxWalls() const {
        if (isD2X()) return MAX_WALLS_D2X;
        return isD1() ? MAX_WALLS_D1 : MAX_WALLS_D2;
    }
    
    int getMaxObjects() const {
        return isD2X() ? MAX_OBJECTS_D2X : MAX_OBJECTS;
    }
    
    int getMaxTriggers() const {
        return isD2X() ? MAX_TRIGGERS_D2X : MAX_TRIGGERS;
    }
    
    // Validation
    bool canAddSegment() const {
        return getSegmentCount() < getMaxSegments();
    }
    
    bool canAddVertex() const {
        return getVertexCount() < getMaxVertices();
    }
    
    // Level properties
    int getReactorTime() const { return m_reactorTime; }
    void setReactorTime(int time) { m_reactorTime = time; }
    
    int getReactorStrength() const { return m_reactorStrength; }
    void setReactorStrength(int strength) { m_reactorStrength = strength; }
    
    int getSecretSegment() const { return m_secretSegment; }
    void setSecretSegment(int segment) { m_secretSegment = segment; }
    
    const Matrix& getSecretOrientation() const { return m_secretOrientation; }
    void setSecretOrientation(const Matrix& orient) { m_secretOrientation = orient; }
    
    // Descent 1 specific
    bool hasHostageText() const { return !m_hostageText.empty(); }
    const std::string& getHostageText() const { return m_hostageText; }
    void setHostageText(const std::string& text) { m_hostageText = text; }
    
    // Change tracking
    bool hasUnsavedChanges() const { return m_changesMade; }
    void setChangesMade(bool changed) { m_changesMade = changed; }
    void markChanged() { m_changesMade = true; }
    void markSaved() { m_changesMade = false; }
    
    // File I/O
    bool load(const std::string& filename);
    bool save(const std::string& filename);
    
    // Create default level (one segment)
    void createDefault();

private:
    // File metadata
    FileType m_fileType;
    int m_levelVersion;
    std::string m_levelName;
    
    // Geometry
    std::vector<Vertex> m_vertices;
    std::vector<Segment> m_segments;
    
    // Level properties
    int m_reactorTime;          // Reactor countdown time (seconds)
    int m_reactorStrength;      // Reactor strength (-1 = invulnerable)
    int m_secretSegment;        // Secret exit segment
    Matrix m_secretOrientation; // Secret exit orientation
    std::string m_hostageText;  // Hostage text (D1 only)
    
    // State
    bool m_changesMade;         // Has the level been modified?
    
    // TODO: Add these later when we implement the corresponding classes:
    // std::vector<Wall> m_walls;
    // std::vector<Trigger> m_triggers;
    // std::vector<Object> m_objects;
    // std::vector<Matcen> m_matcens;
    // std::unique_ptr<Reactor> m_reactor;
};

} // namespace dle

#endif // DLE_MINE_H

#pragma once

#include <QWidget>
#include <memory>

namespace Ui {
class ObjectTool;
}

namespace dle {

class Mine;
class Object;

/**
 * @brief Object editing tool panel
 * 
 * Provides UI for editing objects in the mine:
 * - Robots, hostages, player starts, powerups, reactors
 * - Object properties: type, ID, segment, size, shields
 * - AI settings for robots
 * - Powerup settings
 * - Contents (what robot drops when destroyed)
 * - Actions: Add, Delete, Move, Reset objects
 */
class ObjectTool : public QWidget {
    Q_OBJECT

public:
    explicit ObjectTool(QWidget* parent = nullptr);
    ~ObjectTool() override;

    /**
     * @brief Set the mine data to observe
     * @param mine Pointer to mine (non-owning, can be nullptr)
     */
    void setMine(const dle::Mine* mine);

    /**
     * @brief Update UI from current object data
     */
    void refresh();

signals:
    /**
     * @brief Emitted when object data is modified
     */
    void objectModified();

private slots:
    // Object selection
    void onObjectNumberChanged(int objectNum);
    void onTypeChanged(int index);
    void onIdChanged(int id);
    
    // Properties
    void onSizeChanged(double size);
    void onShieldsChanged(double shields);
    
    // AI settings (for robots)
    void onBehaviorChanged(int behavior);
    void onHideSegmentChanged(int hideSegment);
    void onPathLengthChanged(int pathLength);
    
    // Powerup settings
    void onCountChanged(int count);
    
    // Contents (robot death drop)
    void onContentsTypeChanged(int index);
    void onContentsIdChanged(int id);
    void onContentsCountChanged(int count);
    
    // Actions
    void onAddClicked();
    void onDeleteClicked();
    void onMoveClicked();
    void onResetClicked();
    void onDeleteAllClicked();

private:
    /**
     * @brief Update UI controls from current object
     */
    void updateFromObject();
    
    /**
     * @brief Get current object being edited
     * @return Pointer to object or nullptr if invalid
     */
    dle::Object* getCurrentObject();
    const dle::Object* getCurrentObject() const;
    
    /**
     * @brief Enable/disable controls based on current state
     */
    void updateControlStates();
    
    /**
     * @brief Map object type enum to combo box index
     */
    int objectTypeToIndex(int8_t type) const;
    
    /**
     * @brief Map combo box index to object type enum
     */
    int8_t indexToObjectType(int index) const;
    
    /**
     * @brief Map contents type to combo box index
     */
    int contentsTypeToIndex(int8_t type) const;
    
    /**
     * @brief Map combo box index to contents type
     */
    int8_t indexToContentsType(int index) const;

private:
    std::unique_ptr<Ui::ObjectTool> ui;
    const Mine* m_mine;  ///< Non-owning observer pointer
    bool m_updating;     ///< Flag to prevent update recursion
};

} // namespace dle

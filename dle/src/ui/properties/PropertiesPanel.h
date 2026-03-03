#pragma once

#include <QWidget>
#include <QLabel>
#include <QVBoxLayout>
#include <QGroupBox>
#include <QScrollArea>
#include <memory>

namespace dle {

// Forward declarations
class Mine;
class Segment;
class Wall;
class Trigger;
class Object;

/**
 * PropertiesPanel - Context-sensitive read-only property viewer
 * 
 * Displays quick-view information about the current selection without
 * requiring tab switching. Shows different content based on selection type:
 * - Segment: ID, type, connections, special properties
 * - Wall: Type, keys, flags, clip number
 * - Trigger: Type, targets, flags
 * - Object: Type, ID, AI settings
 * 
 * This complements the editable tool tabs by providing at-a-glance info.
 */
class PropertiesPanel : public QWidget {
    Q_OBJECT

public:
    explicit PropertiesPanel(QWidget* parent = nullptr);
    ~PropertiesPanel() override;

    /**
     * Set the mine data source
     * @param mine Pointer to the mine (non-owning observation pointer)
     */
    void setMine(const Mine* mine);

    /**
     * Refresh the panel to show current selection properties
     */
    void refresh();

    /**
     * Update display based on current selection
     * Called when selection changes in the editor
     */
    void updateSelection();

private:
    void setupUI();
    void clearDisplay();
    void showSegmentProperties();
    void showWallProperties();
    void showTriggerProperties();
    void showObjectProperties();
    void showNoSelection();

    // Helper to format property displays
    QString formatProperty(const QString& label, const QString& value);

    const Mine* m_mine = nullptr;

    // UI Components
    QScrollArea* m_scrollArea = nullptr;
    QWidget* m_contentWidget = nullptr;
    QVBoxLayout* m_contentLayout = nullptr;
    QLabel* m_titleLabel = nullptr;
    QLabel* m_infoLabel = nullptr;
};

} // namespace dle

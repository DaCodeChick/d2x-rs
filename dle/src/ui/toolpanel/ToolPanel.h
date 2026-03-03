#pragma once

#include <QDockWidget>
#include <memory>

QT_BEGIN_NAMESPACE
class QTabWidget;
QT_END_NAMESPACE

namespace dle {

class Mine;
class SegmentTool;
class WallTool;
class TriggerTool;
class ObjectTool;
class TextureTool;
class DiagnosticsTool;

/**
 * @brief Tool Panel - Main editing tools container
 * 
 * Provides tabbed interface for all level editing operations.
 * Replicates the original DLE's CPropertySheet-based tool panel.
 * 
 * Tabs:
 * - Segment: Edit segment properties, sides, vertices
 * - Wall: Configure walls, doors, keys, triggers
 * - Trigger: Set up triggers and their targets
 * - Object: Place and configure objects (robots, powerups, etc.)
 * - Texture: Texture alignment and lighting
 * - Diagnostics: Level validation and statistics
 */
class ToolPanel : public QDockWidget {
    Q_OBJECT

public:
    explicit ToolPanel(QWidget *parent = nullptr);
    ~ToolPanel();

    /**
     * @brief Set the mine data for all tools
     * @param mine Pointer to the mine being edited
     */
    void setMine(Mine* mine);

    /**
     * @brief Refresh all tools with current mine data
     */
    void refreshAll();

    /**
     * @brief Switch to specific tool tab
     * @param tabName Name of the tab ("Segment", "Wall", etc.)
     */
    void showTab(const QString& tabName);

public slots:
    /**
     * @brief Called when current tab changes
     * @param index New tab index
     */
    void onTabChanged(int index);

    /**
     * @brief Refresh the currently visible tab
     */
    void refreshCurrentTab();

signals:
    /**
     * @brief Emitted when tool selection changes
     * @param toolName Name of the activated tool
     */
    void toolChanged(const QString& toolName);

private:
    void setupUi();
    void setupConnections();

    QTabWidget* m_tabWidget;
    Mine* m_mine;

    // Tool tabs
    std::unique_ptr<SegmentTool> m_segmentTool;
    std::unique_ptr<WallTool> m_wallTool;
    std::unique_ptr<TriggerTool> m_triggerTool;
    std::unique_ptr<ObjectTool> m_objectTool;
    std::unique_ptr<TextureTool> m_textureTool;
    std::unique_ptr<DiagnosticsTool> m_diagnosticsTool;
};

} // namespace dle

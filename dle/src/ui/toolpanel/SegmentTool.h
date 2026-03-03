#pragma once

#include <QWidget>
#include <memory>

namespace Ui {
class SegmentTool;
}

namespace dle {

class Mine;
class Segment;

/**
 * @brief Segment editing tool panel
 * 
 * Provides controls for editing segment properties including:
 * - Segment ID selection
 * - Function/type (matcen, goal, etc.)
 * - Properties (water, lava, fog, etc.)
 * - Lighting
 * - Basic operations (add, delete, split)
 */
class SegmentTool : public QWidget {
    Q_OBJECT

public:
    explicit SegmentTool(QWidget *parent = nullptr);
    ~SegmentTool();

    /**
     * @brief Set the mine data source (non-owning observer)
     */
    void setMine(const Mine* mine);

    /**
     * @brief Refresh the tool with current mine data
     */
    void refresh();

signals:
    /**
     * @brief Emitted when a segment property is modified
     */
    void segmentModified(int segmentId);

private slots:
    void onSegmentIdChanged(int value);
    void onFunctionChanged(int index);
    void onLightChanged(int value);
    void onPropertyToggled(bool checked);
    void onAddSegment();
    void onDeleteSegment();
    void onSplitSegment7();
    void onSplitSegment8();

private:
    void setupConnections();
    void updateDisplay();
    void enableControls(bool enable);
    
    std::unique_ptr<Ui::SegmentTool> ui;
    const Mine* m_mine;
    int m_currentSegmentId;
};

} // namespace dle
